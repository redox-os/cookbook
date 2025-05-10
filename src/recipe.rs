use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use serde::{
    de::{Error as DeError, Unexpected},
    Deserialize, Deserializer, Serialize,
};

use crate::recipe_find::recipe_find;

/// Specifies how to download the source for a recipe
#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum SourceRecipe {
    /// Reuse the source directory of another package
    ///
    /// This is useful when a single source repo contains multiple projects which each have their
    /// own recipe to build them.
    SameAs {
        /// Relative path to the package for which to reuse the source dir
        same_as: String,
    },
    /// Path source
    Path {
        /// The path to the source
        path: String,
    },
    /// A git repository source
    Git {
        /// The URL for the git repository, such as https://gitlab.redox-os.org/redox-os/ion.git
        git: String,
        /// The URL for an upstream repository
        upstream: Option<String>,
        /// The optional branch of the git repository to track, such as master. Please specify to
        /// make updates to the rev easier
        branch: Option<String>,
        /// The optional revision of the git repository to use for builds. Please specify for
        /// reproducible builds
        rev: Option<String>,
        /// A list of patch files to apply to the source
        #[serde(default)]
        patches: Vec<String>,
        /// Optional script to run to prepare the source
        script: Option<String>,
    },
    /// A tar file source
    Tar {
        /// The URL of a tar source
        tar: String,
        /// The optional blake3 sum of the tar file. Please specify this to make reproducible
        /// builds more reliable
        blake3: Option<String>,
        /// A list of patch files to apply to the source
        #[serde(default)]
        patches: Vec<String>,
        /// Optional script to run to prepare the source, such as ./autogen.sh
        script: Option<String>,
    },
}

// Deserialize a [`SourceRecipe`] that's valid for mirrors (no local sources).
fn mirror_de<'de, D>(deserializer: D) -> Result<HashMap<String, SourceRecipe>, D::Error>
where
    D: Deserializer<'de>,
{
    let mirrors = HashMap::deserialize(deserializer)?;
    // Sanitize mirrors.
    // Moving Git and Tar into its own type would be cleaner but then it's harder to work with
    // functions that need a &SourceRecipe when we only have the new type.
    for source in mirrors.values() {
        if !matches!(source, SourceRecipe::Git { .. } | SourceRecipe::Tar { .. }) {
            return Err(DeError::invalid_type(
                Unexpected::Other("local source (same as, path)"),
                &"remote source (git, tar)",
            ));
        }
    }

    Ok(mirrors)
}

/// Specifies how to build a recipe
#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "template")]
pub enum BuildKind {
    /// Will build and install using cargo
    #[serde(rename = "cargo")]
    Cargo {
        #[serde(default)]
        package_path: Option<String>,

        #[serde(default)]
        cargoflags: String,
    },
    /// Will build and install using configure and make
    #[serde(rename = "configure")]
    Configure,
    /// Will build and install using custom commands
    #[serde(rename = "custom")]
    Custom { script: String },
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct BuildRecipe {
    #[serde(flatten)]
    pub kind: BuildKind,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct PackageRecipe {
    #[serde(default)]
    pub dependencies: Vec<String>,
}

/// Everything required to build a Redox package
#[derive(Debug, PartialEq, Serialize)]
pub struct Recipe {
    /// Specifies how to download the source for this recipe
    pub source: Option<SourceRecipe>,
    #[serde(default, deserialize_with = "mirror_de", rename = "mirror")]
    pub mirrors: HashMap<String, SourceRecipe>,
    /// Specifies how to build this recipe
    pub build: BuildRecipe,
    /// Specifies how to package this recipe
    #[serde(default)]
    pub package: PackageRecipe,
}

impl<'de> Deserialize<'de> for Recipe {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Delegate {
            pub source: Option<SourceRecipe>,
            #[serde(default, deserialize_with = "mirror_de", rename = "mirror")]
            pub mirrors: HashMap<String, SourceRecipe>,
            pub build: BuildRecipe,
            #[serde(default)]
            pub package: PackageRecipe,
        }

        let mut recipe = Delegate::deserialize(deserializer)?;

        // Clone any source scripts so recipe writers don't have to provide them per mirror.
        for mirror in recipe.mirrors.values_mut() {
            if let (
                Some(SourceRecipe::Git {
                    patches, script, ..
                })
                | Some(SourceRecipe::Tar {
                    patches, script, ..
                }),
                SourceRecipe::Git {
                    patches: patches_m,
                    script: script_m,
                    ..
                }
                | SourceRecipe::Tar {
                    patches: patches_m,
                    script: script_m,
                    ..
                },
            ) = (recipe.source.as_ref(), mirror)
            {
                // Don't clobber if mirror has a patch/script
                if patches_m.is_empty() {
                    patches_m.extend(patches.iter().cloned());
                }
                if let Some(script) = script.as_deref() {
                    script_m.get_or_insert_with(|| script.to_owned());
                }
            }
        }

        let Delegate {
            source,
            mirrors,
            build,
            package,
        } = recipe;
        Ok(Recipe {
            source,
            mirrors,
            build,
            package,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct CookRecipe {
    pub name: String,
    pub dir: PathBuf,
    pub recipe: Recipe,
}

impl CookRecipe {
    pub fn new(name: String) -> Result<Self, String> {
        //TODO: sanitize recipe name?
        let dir = recipe_find(&name, Path::new("recipes"))?;
        if dir.is_none() {
            return Err(format!("failed to find recipe directory '{}'", name));
        }
        let dir = dir.unwrap();
        let file = dir.join("recipe.toml");
        if !file.is_file() {
            return Err(format!("failed to find recipe file '{}'", file.display()));
        }

        let toml = fs::read_to_string(&file).map_err(|err| {
            format!(
                "failed to read recipe file '{}': {}\n{:#?}",
                file.display(),
                err,
                err
            )
        })?;

        let recipe: Recipe = toml::from_str(&toml).map_err(|err| {
            format!(
                "failed to parse recipe file '{}': {}\n{:#?}",
                file.display(),
                err,
                err
            )
        })?;

        Ok(Self { name, dir, recipe })
    }

    pub fn new_recursive(names: &[String], recursion: usize) -> Result<Vec<Self>, String> {
        if recursion == 0 {
            return Err(format!(
                "recursion limit while processing build dependencies: {:#?}",
                names
            ));
        }

        let mut recipes = Vec::new();
        for name in names {
            let recipe = Self::new(name.clone())?;

            let dependencies =
                Self::new_recursive(&recipe.recipe.build.dependencies, recursion - 1).map_err(
                    |err| format!("{}: failed on loading build dependencies:\n{}", name, err),
                )?;

            for dependency in dependencies {
                if !recipes.contains(&dependency) {
                    recipes.push(dependency);
                }
            }

            if !recipes.contains(&recipe) {
                recipes.push(recipe);
            }
        }

        Ok(recipes)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::recipe::{BuildKind, BuildRecipe, PackageRecipe, Recipe, SourceRecipe};

    #[test]
    fn git_cargo_recipe() {
        let recipe: Recipe = toml::from_str(
            r#"
            [source]
            git = "https://gitlab.redox-os.org/redox-os/acid.git"
            branch = "master"
            rev = "06344744d3d55a5ac9a62a6059cb363d40699bbc"

            [build]
            template = "cargo"
        "#,
        )
        .unwrap();

        assert_eq!(
            recipe,
            Recipe {
                source: Some(SourceRecipe::Git {
                    git: "https://gitlab.redox-os.org/redox-os/acid.git".to_string(),
                    upstream: None,
                    branch: Some("master".to_string()),
                    rev: Some("06344744d3d55a5ac9a62a6059cb363d40699bbc".to_string()),
                    patches: Vec::new(),
                    script: None,
                }),
                mirrors: HashMap::default(),
                build: BuildRecipe {
                    kind: BuildKind::Cargo {
                        package_path: None,
                        cargoflags: String::new(),
                    },
                    dependencies: Vec::new(),
                },
                package: PackageRecipe {
                    dependencies: Vec::new(),
                },
            }
        );
    }

    #[test]
    fn tar_custom_recipe() {
        let recipe: Recipe = toml::from_str(
            r#"
            [source]
            tar = "http://downloads.xiph.org/releases/ogg/libogg-1.3.3.tar.xz"
            blake3 = "8220c0e4082fa26c07b10bfe31f641d2e33ebe1d1bb0b20221b7016bc8b78a3a"

            [build]
            template = "custom"
            script = "make"
        "#,
        )
        .unwrap();

        assert_eq!(
            recipe,
            Recipe {
                source: Some(SourceRecipe::Tar {
                    tar: "http://downloads.xiph.org/releases/ogg/libogg-1.3.3.tar.xz".to_string(),
                    blake3: Some(
                        "8220c0e4082fa26c07b10bfe31f641d2e33ebe1d1bb0b20221b7016bc8b78a3a"
                            .to_string()
                    ),
                    patches: Vec::new(),
                    script: None,
                }),
                mirrors: HashMap::default(),
                build: BuildRecipe {
                    kind: BuildKind::Custom {
                        script: "make".to_string()
                    },
                    dependencies: Vec::new(),
                },
                package: PackageRecipe {
                    dependencies: Vec::new(),
                },
            }
        );
    }

    #[test]
    fn recipe_with_mirrors() {
        let tar = "https://github.com/ZDoom/gzdoom/archive/refs/tags/g4.14.1.zip".to_owned();
        let blake3 =
            Some("17860b1d4cb1d26b0ccd34364977d32c098c75c2a6d3c85831774448210c754f".to_owned());
        let github = "https://github.com/ZDoom/gzdoom/".to_owned();
        let branch = Some("master".to_owned());
        let rev = Some("99aa489d09015a95bb78df2b30ede29f328cc874".to_owned());
        let redox = "https://gitlab.redox-os.org/josh/gzdoom_mirror.git".to_owned();
        let script = "cmake".to_owned();

        let recipe: Recipe = toml::from_str(&format!(
            r#"
            [source]
            tar = {:?}
            blake3 = {:?}

            [mirror.github]
            git = {:?}
            branch = {:?}
            rev = {:?}

            [mirror.redox]
            git = {:?}

            [build]
            template = "custom"
            script = {:?}
            "#,
            tar,
            blake3.clone().unwrap(),
            github,
            branch.clone().unwrap(),
            rev.clone().unwrap(),
            redox,
            script,
        ))
        .unwrap();

        let mirrors = vec![
            (
                "github".to_owned(),
                SourceRecipe::Git {
                    git: github,
                    upstream: None,
                    branch,
                    rev,
                    patches: Vec::default(),
                    script: None,
                },
            ),
            (
                "redox".to_owned(),
                SourceRecipe::Git {
                    git: redox,
                    upstream: None,
                    branch: None,
                    rev: None,
                    patches: Vec::default(),
                    script: None,
                },
            ),
        ]
        .into_iter()
        .collect();

        assert_eq!(
            Recipe {
                source: Some(SourceRecipe::Tar {
                    tar,
                    blake3,
                    patches: Vec::default(),
                    script: None
                }),
                mirrors,
                build: BuildRecipe {
                    kind: BuildKind::Custom { script },
                    dependencies: Vec::default()
                },
                package: PackageRecipe {
                    dependencies: Vec::default()
                }
            },
            recipe
        )
    }

    #[test]
    #[should_panic]
    fn recipe_with_invalid_mirror() {
        let _: Recipe = toml::from_str(
            r#"
            [source]
            git = "https://github.com/opentyrian/opentyrian"

            [mirror.invalid]
            path = "/opt/sources/tyrian"

            [build]
            template = "custom"
            script = {:?}
            "#,
        )
        .expect("Non-local mirror should fail");
    }

    #[test]
    fn recipe_with_mirrors_and_single_script() {
        let github = "https://github.com/dolphin-emu/dolphin";
        let redox = "https://gitlab.redox-os.org/josh/dolphin_mirror.git";
        let patches = vec!["redox1.patch".to_owned(), "redox2.patch".to_owned()];
        let script = "./importantstuff.sh";
        let mirrors = vec![(
            "redox".to_owned(),
            SourceRecipe::Git {
                git: redox.to_owned(),
                upstream: None,
                branch: None,
                rev: None,
                patches: patches.clone(),
                script: Some(script.to_owned()),
            },
        )]
        .into_iter()
        .collect();

        let recipe: Recipe = toml::from_str(&format!(
            r#"
            [source]
            git = {:?}
            patches = {:?}
            script = {:?}

            [mirror.redox]
            git = {:?}

            [build]
            template = "configure"
            "#,
            github, patches, script, redox,
        ))
        .unwrap();

        assert_eq!(
            Recipe {
                source: Some(SourceRecipe::Git {
                    git: github.to_owned(),
                    upstream: None,
                    branch: None,
                    rev: None,
                    patches,
                    script: Some(script.to_owned())
                }),
                mirrors,
                build: BuildRecipe {
                    kind: BuildKind::Configure,
                    dependencies: Vec::default()
                },
                package: PackageRecipe {
                    dependencies: Vec::default()
                }
            },
            recipe
        )
    }

    #[test]
    fn recipe_with_mirrors_and_mirror_scripts() {
        let github = "https://github.com/mgba-emu/mgba";
        let patches_main = vec!["redox1.patch".to_owned()];
        let script_main = "./foobar.sh";

        let redox = "https://gitlab.redox-os.org/josh/mgba_mirror.git";
        let patches_mirror = vec!["mirror.patch".to_owned()];
        let script_mirror = "";
        let mirrors = vec![(
            "redox".to_owned(),
            SourceRecipe::Git {
                git: redox.to_owned(),
                upstream: None,
                branch: None,
                rev: None,
                patches: patches_mirror.clone(),
                script: Some(script_mirror.to_owned()),
            },
        )]
        .into_iter()
        .collect();

        let recipe: Recipe = toml::from_str(&format!(
            r#"
            [source]
            git = {:?}
            patches = {:?}
            script = {:?}

            [mirror.redox]
            git = {:?}
            patches = {:?}
            script = {:?}

            [build]
            template = "configure"
            "#,
            github, patches_main, script_main, redox, patches_mirror, script_mirror
        ))
        .unwrap();

        assert_eq!(
            Recipe {
                source: Some(SourceRecipe::Git {
                    git: github.to_owned(),
                    upstream: None,
                    branch: None,
                    rev: None,
                    patches: patches_main,
                    script: Some(script_main.to_owned())
                }),
                mirrors,
                build: BuildRecipe {
                    kind: BuildKind::Configure,
                    dependencies: Vec::default()
                },
                package: PackageRecipe {
                    dependencies: Vec::default()
                }
            },
            recipe
        );
    }
}

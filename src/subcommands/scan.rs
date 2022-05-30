use std::{ffi::OsStr, fs, sync::Arc};

use anyhow::Result;
use colored::Colorize;
use ferinth::Ferinth;
use furse::Furse;
use itertools::Itertools;
use libium::{
    config::structs::{Mod, ModIdentifier, ModPlatform, Profile},
    scan,
};

use crate::{CROSS, TICK, YELLOW_TICK};
pub async fn scan(
    modrinth: Arc<Ferinth>,
    curseforge: Arc<Furse>,
    profile: &mut Profile,
    preferred_platform: libium::config::structs::ModPlatform,
) -> Result<()> {
    for mod_file in fs::read_dir(&profile.output_dir)? {
        let mod_path = mod_file?.path();
        if matches!(mod_path.extension().and_then(OsStr::to_str), Some("jar")) {
            match libium::scan::scan(modrinth.clone(), curseforge.clone(), &mod_path).await {
                Ok(mods) => {
                    let mod_to_add =
                        mods.iter()
                            .find_or_first(|mod_| match (&preferred_platform, mod_) {
                                (ModPlatform::Curseforge, ModIdentifier::CurseForgeProject(_))
                                | (ModPlatform::Modrinth, ModIdentifier::ModrinthProject(_)) => {
                                    true
                                },
                                _ => false,
                            });
                    match mod_to_add {
                        Some(ModIdentifier::ModrinthProject(id)) => {
                            let result =
                                libium::add::modrinth(modrinth.clone(), &id, profile, None, None)
                                    .await;
                            // make sure it doesn't crash if the mod is already added
                            if !matches!(result, Err(libium::add::Error::AlreadyAdded)) {
                                let (project, _version) = result?;
                                println!("{} found {} on Modrinth", TICK.clone(), project.title);
                                profile.mods.push(Mod {
                                    check_game_version: None,
                                    check_mod_loader: None,
                                    identifier: ModIdentifier::ModrinthProject(project.id),
                                    name: project.title,
                                })
                            } else {
                                println!(
                                    "{} {} is already added",
                                    YELLOW_TICK.clone(),
                                    mod_path.display()
                                )
                            }
                        },
                        Some(ModIdentifier::CurseForgeProject(id)) => {
                            let result = libium::add::curseforge(
                                curseforge.clone(),
                                *id,
                                profile,
                                None,
                                None,
                            )
                            .await;
                            if !matches!(result, Err(libium::add::Error::AlreadyAdded)) {
                                let (project, _file) = result?;
                                println!(
                                    "{} found mod {} on CurseForge",
                                    TICK.clone(),
                                    project.name
                                );
                                profile.mods.push(Mod {
                                    check_game_version: None,
                                    check_mod_loader: None,
                                    identifier: ModIdentifier::CurseForgeProject(project.id),
                                    name: project.name,
                                })
                            } else {
                                println!(
                                    "{} {} is already added",
                                    YELLOW_TICK.clone(),
                                    mod_path.display()
                                )
                            }
                        },
                        _ => unreachable!(),
                    }
                },
                Err(scan::Error::DoesNotExist) => {
                    eprintln!(
                        "{}",
                        format!(
                            "{} Could not find {} on any platform",
                            CROSS,
                            mod_path.display()
                        )
                        .red()
                    );
                    continue;
                },
                Err(err) => {
                    return Err(err.into());
                },
            }
        }
    }
    Ok(())
}

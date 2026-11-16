# Zenda build of PhotoCraft for Work

Based on upstream v0.23.1. Feature `zenda` (crate `photocraft-ui-egui`, forwarded by the
`photocraft` app) adds:

- a bounded `work.events` queue on the control channel, drained by Work after it authenticates
  with the 64-hex token it passes in `PHOTOCRAFT_CONTROL_TOKEN`;
- the orange Theo mascot button at the far end of the title bar (`work.theo`, from `button`);
  Home › Learn asks Theo too (`work.theo`, from `learn`) instead of opening ArtCraft's tutorials;
- File › Share… (`zenda.share`), which queues `work.share` with the image's path, name and unsaved
  state. Work does the sending; the document format is unchanged.

Strip: no upstream update checks (the startup check never runs and Help › Check for Updates is
gone), no Discord, PhotoCraft website or ArtCraft website items in Help, no Website or Discord
links on the Home screen; GitHub and Report an Issue point at this fork.

Brand (`docs/brand/LICENSE-brand.txt`): the app embeds no ArtCraft Marks. About says
«Basada en PhotoCraft, del equipo de ArtCraft» and links to this fork. The kitsune app icon is
PhotoCraft's own, not an ArtCraft Mark, and stays.

Theo's image is the existing Zenda Work mascot (`app/ui/theo/theo-icon-64.png`, SHA-256
`87770f80…bbeb4`). Button: #EA6C1E with black text and a localized AccessKit name (es/en/pt-BR).

Translations: `zenda/i18n/<locale>/photocraft.tsv` rows are appended to craft-i18n at the commit
in `craft-i18n.rev` by `zenda/i18n.ps1` before the build. Spanish is 100 % of PhotoCraft's
strings (`cargo xtask i18n-report` with that checkout as `CRAFT_I18N_DIR`).

Packaging: `pwsh zenda/package.ps1 -Z 1`; workflow `Zenda` publishes immutable
`zenda-v0.23.1-z<n>` GitHub releases. Verify size and SHA-256 before editing Work's manifest.

## Checks

`cargo test --config 'profile.dev.package.photocraft-ui-egui.opt-level=0' -p photocraft-ui-egui --lib --features zenda zenda::`
and the same without the feature.

# Dandy Harness v0.1.0 — préversion autonome

Les installateurs contiennent Tauri, Node 24.22.3 et le moteur. Aucun checkout
du dépôt, pnpm ou installation séparée de Node n'est nécessaire à l'utilisation.

| Système | Fichier | Vérification |
| --- | --- | --- |
| Windows x64 | `Dandy-Harness_0.1.0_x64-setup.exe` | Installation silencieuse et moteur installé vérifiés |
| macOS Apple Silicon | `Dandy-Harness_0.1.0_aarch64.dmg` | DMG monté en lecture seule, signature locale et moteur vérifiés |
| macOS Intel | Non disponible | Échec du packaging : expiration de DiskArbitration lors du démontage du DMG |
| Linux x64 Debian/Ubuntu | `Dandy-Harness_0.1.0_amd64.deb` | Extraction et moteur du paquet vérifiés |
| Linux x64 AppImage | `Dandy-Harness_0.1.0_amd64.AppImage` | Extraction et moteur du paquet vérifiés |

Fermer l'ancienne application avant installation. Sur macOS, glisser l'application
vers Applications. Sur Debian/Ubuntu :

```sh
sudo apt install ./Dandy-Harness_0.1.0_amd64.deb
```

Pour l'AppImage :

```sh
chmod +x Dandy-Harness_0.1.0_amd64.AppImage
./Dandy-Harness_0.1.0_amd64.AppImage
```

`SHA256SUMS.txt` permet de vérifier les fichiers téléchargés (`sha256sum -c
SHA256SUMS.txt` sous Linux, `Get-FileHash` sous PowerShell).

## Ce qui a été vérifié

- Windows : 11 tests Rust réussis, Clippy, NSIS installé puis moteur embarqué HTTP 200.
- Apple Silicon et Linux : 9 tests Rust réussis par plateforme, Clippy, moteur
  testé avant et après empaquetage. Un benchmark est ignoré sur chaque plateforme.
- Linux : test du filtrage des prébuilds incompatibles ; variantes glibc, terminal
  natif correspondant et lanceur statique du sandbox conservés.
- Le chat et la lecture d'un fichier avec Glimmer ont été testés dans Tauri sur
  Windows avant livraison. Les parcours graphiques interactifs macOS et Linux
  n'ont pas été vérifiés ; les essais CI ne les remplacent pas.

L'EXE n'a pas de certificat éditeur. Les DMG sont signés ad hoc, sans notarisation
Apple. Les systèmes peuvent donc afficher un avertissement de réputation ou
demander une autorisation d'ouverture. Il s'agit d'une préversion de test.

## Sources et provenance

Moteur épinglé : `dandyArise/deepseek-harness@f0a26e7f33f5f69619c3793f61f2e4d6909ee3aa`.
Code applicatif du lanceur : tag `v0.1.0` / commit `b1e6e72`.

- [Windows — build et installation vérifiés](https://github.com/dandyArise/dandy-harness/actions/runs/45203171703), scripts `11bc820`.
- [Apple Silicon — DMG vérifié](https://github.com/dandyArise/dandy-harness/actions/runs/45214760172), scripts `b1e6e72`.
- [Linux — DEB et AppImage vérifiés](https://github.com/dandyArise/dandy-harness/actions/runs/45218262270), scripts `9201fc6`.
- [Intel — packaging échoué](https://github.com/dandyArise/dandy-harness/actions/runs/45214768025) : tests Rust, Clippy et compilation réussis ; `hdiutil detach` expire après la création de l'image temporaire. Aucun DMG Intel n'est publié.

Les différences entre ces scripts concernent le packaging et les contrôles, pas
le code applicatif Rust, les ressources de l'interface ou la version du moteur.
Pour reproduire le packaging Linux, utiliser `9201fc6` ou une révision suivante
contenant le filtre musl de Koffi. Le workflow `Installers` permet de sélectionner
`all`, `windows`, `macos-arm64`, `macos-x64` ou `linux`.

Les données, conversations, clés et réglages de modèles sont conservés hors des
installateurs. Pour un ancien dossier de données personnalisé, `DSH_HOME` permet
de conserver cet emplacement ; ne pas supprimer les données de l'ancienne version.

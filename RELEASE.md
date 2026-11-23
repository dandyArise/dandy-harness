# Dandy Harness 0.1.0 — installateurs autonomes

Le moteur et Node sont inclus. Aucun checkout du code ni installation de Node
n'est nécessaire pour utiliser l'application. Configurez votre fournisseur LLM
dans Harness ; aucune clé API ni conversation personnelle n'est embarquée.

## Quel fichier choisir ?

- Windows x64 : `Dandy Harness_0.1.0_x64-setup.exe`.
- Mac Apple Silicon (M1 et suivants) : le `.dmg` contenant `aarch64`.
- Mac Intel : le `.dmg` contenant `x64`.
- Linux x64 Debian/Ubuntu : le `.deb` (installer avec `sudo apt install ./fichier.deb`).
- Autres Linux x64 compatibles : `.AppImage` ; rendre exécutable avec `chmod +x`.

Les builds macOS et Linux constituent une première livraison de test. Chaque
runner compile le moteur et le lanceur, exécute les tests Rust et Clippy, puis
vérifie que le moteur embarqué sert le chat avec un profil neuf. Le parcours
interactif Glimmer et la lecture d'un fichier ont été vérifiés sous Windows ;
ils ne sont pas présentés comme testés manuellement sur Mac/Linux.

Windows n'a pas de certificat Authenticode. macOS utilise une signature ad-hoc,
sans notarisation Apple : une autorisation explicite dans Réglages système →
Confidentialité et sécurité peut être nécessaire. Ne désactivez pas globalement
Gatekeeper. Vérifiez les fichiers avec `SHA256SUMS.txt`.

Les correctifs incluent la reprise d'une conversation dont l'ancien identifiant
de modèle a disparu, le maintien du fournisseur lors du compactage et les tests
de liens Windows. Les données existantes restent conservées.

Sources du packaging : [Tauri resources](https://v2.tauri.app/develop/resources/)
et [Tauri GitHub builds](https://v2.tauri.app/distribute/pipelines/github/).

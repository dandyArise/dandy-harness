# WebView2 : mémoire et discussion libre — 21 novembre 2026

## Observation et modification

Sur l'accueil vide, WebView2 occupait environ 367 Mio privés, dont 187 Mio pour
le processus GPU et 116 Mio pour le renderer. Cette mesure ne démontre pas une
fuite liée aux historiques. La taille de fenêtre influence fortement ces valeurs.

Le lanceur demande maintenant `MemoryUsageTargetLevel.Low` uniquement lorsque
la fenêtre est minimisée, puis `Normal` à sa restauration. Aucun script n'est
suspendu, aucune session n'est rechargée, le GPU reste activé. Une ancienne
version de WebView2 sans cette interface conserve le comportement antérieur.

Référence : [recommandations Microsoft](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/performance).

## Comparaison alternée A/B

Même exe debug, même profil vide et même taille de fenêtre. A désactive le changement
via `DANDY_WEBVIEW_MEMORY=normal`; B utilise le nouveau comportement par défaut.
Deux lancements frais par variante, dans l'ordre A1/B1/A2/B3. B2 était un second
cycle du même processus et est exclu. Les essais perturbés par des clics et
redimensionnements sont exclus. Moyenne des cinq dernières observations parmi
douze échantillons espacés d'une seconde après minimisation :

| Essai | WebView2 privé (Mio) | Somme des working sets (Mio) |
|---|---:|---:|
| A1 | 324,7 | 566,8 |
| B1 | 344,6 | 476,7 |
| A2 | 348,8 | 586,4 |
| B3 | 347,1 | 431,4 |

Réduction résidente observée : 90,1 Mio (-15,9 %) puis 155,0 Mio (-26,4 %).
**Aucun gain reproductible de mémoire privée démontré.** Les working sets peuvent
compter des pages partagées plusieurs fois : ce n'est pas la RAM physique exclusive.
Le temps de chauffe n'était pas strictement identique et il n'y a que deux paires.
Ces résultats ne prouvent ni un gain en fenêtre active ni la correction d'une fuite.

Données : `benchmarks/results/webview-memory-2026-11-21.json` (48 observations).
Reproduction : lancer un seul Harness, garder profil et dimensions identiques,
minimiser, puis exécuter :

```powershell
./benchmarks/webview-memory.ps1 -Label A -Samples 12 -Output ./target/memory-A.json
```

Pour A, définir `$env:DANDY_WEBVIEW_MEMORY='normal'` avant le lancement ; pour B,
retirer cette variable avant le lancement. Alterner les lancements. Les marqueurs
diagnostiques `memory_low` et `memory_normal` ont confirmé les transitions COM.

## Vérifications

- Rust : fmt, Clippy avec `-D warnings`, build, tests : 8 réussis, 1 benchmark ignoré.
- Interface : 161 tests workspace/conversation et 105 tests du panneau sous-agents réussis.
- LiveShare : 5 tests réussis ; les deux routes de consignes globales, déjà présentes
  localement, ont été classées owner-only pour débloquer le contrôle de compilation.
- `build:artifacts` : 8 contrôles réussis ; `build:check` : 2 contrôles réussis.
- Tauri réel : Discussion libre créée sans choisir de dossier ; Glimmer v7 répond
  `7 + 5 = 12.` en 7 s. Conversation et réponse conservées après fermeture/relance.
- Second tour envoyé puis fenêtre minimisée : termine en 6 s, visible après restauration.
  Le modèle expose toutefois du texte de réflexion au lieu de respecter la réponse
  exacte demandée. Ce défaut de sortie n'est pas corrigé par l'optimisation mémoire.
- Aucun sous-agent LLM réel lancé lors de cette vérification ; leur couverture est
  celle des tests automatisés du panneau, pas un essai de délégation en production.
- Aucun modèle chargé/déchargé, aucune configuration de fine-tuning modifiée.

## Discussion libre

Le bouton de l'accueil crée une session persistante non groupée avec le preset
`discussion`, sans sélection d'espace. Nouvelle séance conserve ce mode depuis
une discussion. Le preset n'enregistre ni outils de fichiers ni terminal ; il
réutilise les abstractions de session et de compactage existantes. Le moteur garde
son cwd technique par défaut, sans créer ni attacher un espace de travail.

Les modifications restent locales dans les deux dépôts ; aucune publication effectuée.

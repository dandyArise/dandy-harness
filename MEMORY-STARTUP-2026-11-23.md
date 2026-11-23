# Démarrage et mémoire — 23 novembre 2026

## Décision

Pas de nouveau portage Rust : aucun composant supplémentaire n'a un gain de portage démontré. Une variante mémoire Node est disponible **sur activation explicite seulement**, avec `DANDY_NODE_MEMORY=compact`. Elle ajoute `--max-semi-space-size=4` au seul moteur Node ; ni la mémoire totale du tas ni les processus d'outils/MCP ne sont plafonnés par ce réglage.

La réduction de mémoire est reproductible au démarrage, mais le débit des outils en usage prolongé et un nouveau tour LLM réel restent à vérifier. Ne pas l'activer par défaut sur la base de cette seule série.

## Protocole

Lanceur release basé sur e6e95c0adb9545f16769f28bd99df3f260468cf8, plus l'option locale ; moteur et données locaux conservés. Glimmer sélectionné, conversation parent et conversation de sous-agent ouvertes. Aucun changement de modèle ni de serveur. Même exécutable pour les six essais finaux, ordre normal/compact/compact/normal/normal/compact, aucune compilation parallèle. `benchmarks/startup.ps1` mesure le champ éditable après deux frames et additionne les octets privés des descendants (Tauri, Node, WebView2 et autres). Mémoire : moyenne des cinq derniers échantillons, environ cinq secondes après disponibilité. Caches Windows non purgés.

## Résultats finaux

| Essai | Mode | Chat ms | Moteur ms | Node Mio | WebView2 Mio | Total privé Mio |
|---|---|---:|---:|---:|---:|---:|
| 0 | normal | 2754,904 | 2178,112 | 311,53 | 558,19 | 943,06 |
| 1 | compact | 2643,462 | 2179,242 | 194,53 | 599,97 | 867,68 |
| 2 | compact | 2761,608 | 2243,265 | 188,77 | 605,26 | 867,21 |
| 3 | normal | 2545,039 | 2019,010 | 312,11 | 627,22 | 1012,54 |
| 4 | normal | 2451,992 | 1951,481 | 314,20 | 593,75 | 981,27 |
| 5 | compact | 2527,934 | 2019,312 | 192,26 | 594,39 | 859,63 |

Médianes : total **981,27 → 867,21 Mio**, soit **−114,06 Mio (−11,6 %)** ; Node **312,11 → 192,26 Mio**. Chat **2545,039 → 2643,462 ms**, soit **+98,423 ms (+3,9 %)**. Les plages de démarrage se recouvrent : aucun gain de vitesse revendiqué, pas de p95 ni de conclusion statistique forte avec trois essais par variante. Les octets privés ne sont pas la RAM physique exclusive. WebView2 reste le coût principal et fluctue beaucoup.

Une première série de trois lancements a donné 12,74 s puis 2,42 et 2,54 s ; le premier démarrage atypique n'est pas utilisé pour créditer la variante. Un essai exploratoire via NODE_OPTIONS avait ensuite montré la même réduction mémoire ; la série ci-dessus valide le réglage limité au moteur dans le lanceur.

Bruts locaux : `target/perf-20261123-final-0.json` à `target/perf-20261123-final-5.json`. Synthèse : `benchmarks/results/memory-2026-11-23.json`.

## Vérifications et limites

- `cargo +stable-x86_64-pc-windows-msvc build --release --locked --target x86_64-pc-windows-msvc` : réussi.
- `cargo ... fmt --check`, Clippy `-D warnings` : réussis.
- Tests Rust : 8 réussis, 1 benchmark ignoré.
- 144 tests ciblés du moteur avec zone temporaire réduite : exécuteurs d'outils, UI des sous-agents, archivage. Tests automatisés, pas un échange réel avec le fournisseur.
- Six lancements Tauri réels : éditeur détecté éditable, aucun survivant signalé par le benchmark après arrêt.
- Interface ouverte avec Glimmer toujours sélectionné et les deux conversations présentes. Le contrôle visuel a ensuite renvoyé la fenêtre au premier plan plutôt que la cible : saisie manuelle automatisée non validée.
- Aucun service en écoute sur les ports locaux habituels LM Studio/DGX (1234/18089), aucun processus LM Studio visible : **nouveau chat généré et premier outil via LLM non vérifiés**. Ne pas confondre les anciennes réponses affichées avec un retest.
- Les changements locaux du moteur, notamment le popup, n'ont pas été modifiés par cette passe.

Le mode faible mémoire WebView2 n'a pas été forcé sur le chat actif : [Microsoft recommande de rétablir Normal pour une vue active](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/performance). Le comportement existant lors de la minimisation reste inchangé.

## Reproduire

Fermer Harness, puis alterner les modes sans changer le profil, le modèle, la taille de fenêtre ou les données :

```powershell
$env:DANDY_NODE_MEMORY = 'compact' # ou 'normal'
.\benchmarks\startup.ps1 -Exe 'D:\Github\dandy-harness\target\x86_64-pc-windows-msvc\release\dandy-harness.exe' -Output 'target\mesure.json' -Label $env:DANDY_NODE_MEMORY -Runs 1
Remove-Item Env:DANDY_NODE_MEMORY
```

La variable ne concerne que l'environnement du terminal courant. À la fin du test interactif, Harness a été relancé en mode normal ; aucun mode compact permanent n'a été activé.

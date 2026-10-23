# Randbedingungen

**Maßgebliche Quelle für Randbedingungen.**

| ID | Randbedingung | Status | Herkunft |
|---|---|---|---|
| C-01 | Windows ist primäre Plattform | OWNER DECIDED | Mandat 5 |
| C-02 | Alle Abhängigkeiten müssen lizenz- und distributionsfähig sein; keine ungeprüften Fremdassets | OWNER DECIDED | Mandat 5, 9 |
| C-03 | Keine kostenpflichtigen Dienste/Ressourcen ohne Freigabe | OWNER DECIDED | Mandat 9 |
| C-04 | Kein Commit/Push/Release ohne Freigabe | OWNER DECIDED | Mandat 9 |
| C-05 | Keine technische Abhängigkeit zu HA/HASEC | OWNER DECIDED | Mandat 1 |
| C-06 | Produktlizenz: Apache-2.0 für Code, siehe ADR-0009; Kern-Abhängigkeiten nur permissiv | OWNER DECIDED (delegiert, OD-020) | ADR-0009 |
| C-07 | Vorhandene Assets (`assets/`) sind rechtlich ungeklärt (OQ-03) | RESEARCHED | `../04-research/references.md` C3 |
| C-08 | Entwicklungsumgebung: Cloud-Container ohne Windows/GPU; Windows-Tests und GPU-Messungen erfordern Owner-Hardware oder CI | RESEARCHED (Umgebung) | Sitzung |
| C-09 | Recherchezugriff teils eingeschränkt (Proxy); Primärquellen einzelner Anbieter nicht abrufbar | RESEARCHED | `../04-research/references.md` |
| C-10 | Container-Netzwerk: erreichbar raw.githubusercontent.com, pypi.org; gesperrt github.com (Clone/ZIP/API), codeload, sketchfab, quaternius.com, polyhaven.com, huggingface, static.makehumancommunity.org (geprüft 2026-10-23). Zusätzliche Hosts erfordern Änderung der Netzwerk-Allowlist durch den Owner | RESEARCHED | Sitzung |

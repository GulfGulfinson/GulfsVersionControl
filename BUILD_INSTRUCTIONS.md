# Build Instructions

## Wenn Rust noch nicht installiert ist

### Windows

1. Öffne PowerShell als Administrator
2. Lade Rust herunter und installiere es:
   ```powershell
   # Download rustup-init.exe
   Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "$env:TEMP\rustup-init.exe"
   
   # Run installer
   & "$env:TEMP\rustup-init.exe" -y
   
   # Refresh environment
   $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
   ```

3. Schließe PowerShell und öffne es neu
4. Verifiziere die Installation:
   ```powershell
   rustc --version
   cargo --version
   ```

### Alternative: Manuelle Installation

1. Besuche https://rustup.rs/
2. Lade `rustup-init.exe` herunter
3. Führe die Datei aus und folge den Anweisungen
4. Starte das Terminal neu

## Projekt bauen

Sobald Rust installiert ist:

```powershell
# Im Projektverzeichnis
cd C:\Users\pschneid\Documents\proxy\privat\GulfsControlSystem

# Baue das Projekt (Debug-Modus)
cargo build

# Oder baue optimierte Version (Release-Modus)
cargo build --release
```

## GVC ausführen

### Ohne Installation (direkt aus dem Build)

```powershell
# Debug-Version
cargo run --bin gvc -- --help

# Release-Version
.\target\release\gvc.exe --help
```

### Mit Installation

```powershell
# Installiere gvc in ~/.cargo/bin/
cargo install --path gvc-cli

# Jetzt kannst du gvc direkt aufrufen
gvc --help
```

## Tests ausführen

```powershell
# Alle Tests
cargo test

# Tests mit Ausgabe
cargo test -- --nocapture

# Tests für spezifisches Modul
cargo test --package gvc-core
```

## Entwicklung

### Code formatieren

```powershell
cargo fmt
```

### Code-Qualität prüfen (Linter)

```powershell
cargo clippy
```

### Dokumentation generieren

```powershell
cargo doc --open
```

## Schnellstart nach Installation

```powershell
# Repository initialisieren
gvc init

# Datei erstellen
echo "Hello GVC" > test.txt

# Datei hinzufügen
gvc add test.txt

# Commit erstellen
gvc commit -m "Initial commit"

# Historie anzeigen
gvc log
```

## Troubleshooting

### "cargo: command not found" nach Installation

1. Terminal neu starten
2. Pfad prüfen:
   ```powershell
   $env:Path -split ';' | Select-String cargo
   ```
3. Sollte `C:\Users\<username>\.cargo\bin` enthalten

### Build-Fehler

```powershell
# Cache löschen und neu bauen
cargo clean
cargo build
```

### Rust aktualisieren

```powershell
rustup update
```

## Weitere Informationen

- [INSTALLATION.md](INSTALLATION.md) - Detaillierte Installationsanleitung
- [USAGE.md](USAGE.md) - Nutzungsanleitung
- [ARCHITECTURE.md](ARCHITECTURE.md) - Technische Dokumentation


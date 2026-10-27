use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
struct Args {
    #[command(subcommand)]
    command: Task,
}

#[derive(Debug, Subcommand)]
enum Task {
    /// Generate the fixture calibre library.
    Fixtures {
        /// Target library directory (default: tests/fixtures/library).
        #[arg(long, default_value = "tests/fixtures/library")]
        out: PathBuf,
        /// Work directory for generated source and converted files.
        #[arg(long, default_value = "target/fixtures-src")]
        work: PathBuf,
    },
}

#[derive(Debug, Clone, Copy)]
struct Fixture {
    id: u64,
    title: &'static str,
    authors: &'static str,
    language: &'static str,
    tag: &'static str,
    pubdate: &'static str,
    format: &'static str,
    chapters: usize,
    body: &'static str,
}

const CLAIM_DE: &str = "Vertrauen in Organisationen entsteht durch wiederholte Interaktion und gemeinsame Erfahrung.";
const MIRROR_EN: &str = "Trust in organizations grows through repeated interaction and shared experience.";
const FILLER: &str = "Dieser Abschnitt behandelt ein anderes Thema und enthaelt keinen der gesuchten Begriffe.";
const RARE: &str = "Der Sonderbegriff erscheint hier als seltenes Stichwort fuer die kurze Trefferliste.";

fn fixtures() -> Vec<Fixture> {
    vec![
        Fixture {
            id: 1,
            title: "Vertrauen in Organisationen",
            authors: "Anna Beispiel",
            language: "deu",
            tag: "genre.fachbuch",
            pubdate: "2001-01-01",
            format: "epub",
            chapters: 3,
            body: CLAIM_DE,
        },
        Fixture {
            id: 2,
            title: "Trust in Organizations",
            authors: "Clara Example",
            language: "eng",
            tag: "genre.fachbuch",
            pubdate: "2005-01-01",
            format: "epub",
            chapters: 2,
            body: MIRROR_EN,
        },
        Fixture {
            id: 3,
            title: "Vertrauen in Teams",
            authors: "Bernd Beispiel",
            language: "deu",
            tag: "genre.sachbuch",
            pubdate: "2012-01-01",
            format: "epub",
            chapters: 2,
            body: CLAIM_DE,
        },
        Fixture {
            id: 4,
            title: "Vertrauen als PDF",
            authors: "Anna Beispiel",
            language: "deu",
            tag: "genre.sachbuch",
            pubdate: "2015-01-01",
            format: "pdf",
            chapters: 2,
            body: CLAIM_DE,
        },
        Fixture {
            id: 5,
            title: "Vertrauen nur als MOBI",
            authors: "Anna Beispiel",
            language: "deu",
            tag: "genre.fachbuch",
            pubdate: "2018-01-01",
            format: "mobi",
            chapters: 1,
            body: CLAIM_DE,
        },
        Fixture {
            id: 6,
            title: "Vier seltene Stellen",
            authors: "Clara Example",
            language: "deu",
            tag: "genre.fachbuch",
            pubdate: "2020-01-01",
            format: "epub",
            chapters: 4,
            body: RARE,
        },
        Fixture {
            id: 7,
            title: "Dreissig Kapitel",
            authors: "Bernd Beispiel",
            language: "deu",
            tag: "genre.fachbuch",
            pubdate: "2021-01-01",
            format: "epub",
            chapters: 30,
            body: CLAIM_DE,
        },
        Fixture {
            id: 8,
            title: "Ohne Treffer",
            authors: "Clara Example",
            language: "deu",
            tag: "genre.sonstiges",
            pubdate: "2022-01-01",
            format: "epub",
            chapters: 2,
            body: FILLER,
        },
    ]
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command {
        Task::Fixtures { out, work } => generate(&out, &work),
    }
}

fn generate(out: &Path, work: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(out)?;
    std::fs::create_dir_all(work)?;
    for fixture in fixtures() {
        let html = work.join(format!("work-{}.html", fixture.id));
        std::fs::write(&html, html_for(&fixture))?;
        let converted = work.join(format!("work-{}.{}", fixture.id, fixture.format));
        run(
            "ebook-convert",
            &[
                html.to_string_lossy().as_ref(),
                converted.to_string_lossy().as_ref(),
                "--title",
                fixture.title,
                "--authors",
                fixture.authors,
                "--language",
                if fixture.language == "deu" { "de" } else { "en" },
            ],
        )?;
        let id = fixture.id.to_string();
        let pubdate = format!("pubdate:{}", fixture.pubdate);
        run(
            "calibredb",
            &[
                "add",
                "--library-path",
                out.to_string_lossy().as_ref(),
                "--title",
                fixture.title,
                "--authors",
                fixture.authors,
                "--tags",
                fixture.tag,
                "--languages",
                fixture.language,
                converted.to_string_lossy().as_ref(),
            ],
        )?;
        run(
            "calibredb",
            &[
                "set_metadata",
                "--library-path",
                out.to_string_lossy().as_ref(),
                "--field",
                &pubdate,
                &id,
            ],
        )?;
    }
    Ok(())
}

fn html_for(fixture: &Fixture) -> String {
    let mut chapters = String::new();
    for index in 0..fixture.chapters {
        let body = match fixture.id {
            6 => format!("{RARE} Abschnitt {index}."),
            7 => format!("{CLAIM_DE} Kapitel {index}."),
            _ => format!("{0} Abschnitt {index}.", fixture.body),
        };
        chapters.push_str(&format!(
            "<h1>Kapitel {}</h1><p>{}</p>",
            index + 1,
            body
        ));
    }
    format!(
        "<html><head><title>{}</title></head><body>{chapters}</body></html>",
        fixture.title
    )
}

fn run(program: &str, args: &[&str]) -> anyhow::Result<()> {
    let output = ProcessCommand::new(program)
        .args(args)
        .output()
        .map_err(|error| anyhow::anyhow!("{program} could not be executed: {error}"))?;
    if !output.status.success() {
        anyhow::bail!(
            "{program} {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

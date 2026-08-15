//! Reporting. Nothing here decides anything — it renders what `plan` concluded.

use crate::plan::{Entry, Mechanism, Plan, State};
use owo_colors::OwoColorize;

pub fn report(plan: &Plan) {
    let mut current = String::new();

    for entry in &plan.entries {
        if entry.app != current {
            current = entry.app.clone();
            println!("\n{}", current.bold());
        }
        println!("  {}", line(entry));

        for warning in &entry.warnings {
            println!("      {} {}", "warn".yellow(), warning.dimmed());
        }
        if let Some(note) = &entry.note
            && entry.state.is_problem()
        {
            println!("      {} {}", "note".dimmed(), note.dimmed());
        }
    }

    if !plan.declared_checks.is_empty() {
        println!("\n{}", "declared checks".bold());
        for check in &plan.declared_checks {
            // The label is mandatory: without it, [[verify]] would silently promise
            // something this build does not do (ADR 0007, question 4).
            println!(
                "  {} {} — {}",
                "not run in v1".dimmed(),
                check.app.dimmed(),
                check.description.dimmed()
            );
        }
    }
}

fn line(entry: &Entry) -> String {
    let mechanism = format!("{:<8}", entry.mechanism.label());
    match &entry.state {
        State::Ok => format!("{} {} {}", "ok      ".green(), mechanism.dimmed(), entry.what),
        State::Absent => format!(
            "{} {} {}",
            "absent  ".blue(),
            mechanism.dimmed(),
            entry.what
        ),
        State::Drifted(why) => format!(
            "{} {} {} — {why}",
            "drifted ".yellow(),
            mechanism.dimmed(),
            entry.what
        ),
        State::Conflict(why) => format!(
            "{} {} {} — {why}",
            "conflict".red(),
            mechanism.dimmed(),
            entry.what
        ),
        State::Skipped(why) => format!(
            "{} {} {} — {why}",
            "skipped ".dimmed(),
            mechanism.dimmed(),
            entry.what.dimmed()
        ),
    }
}

pub fn summary(plan: &Plan) {
    let count = |f: fn(&State) -> bool| plan.entries.iter().filter(|e| f(&e.state)).count();
    let ok = count(|s| matches!(s, State::Ok));
    let absent = count(|s| matches!(s, State::Absent));
    let drifted = count(|s| matches!(s, State::Drifted(_)));
    let conflict = count(|s| matches!(s, State::Conflict(_)));
    let skipped = count(|s| matches!(s, State::Skipped(_)));

    println!(
        "\n{ok} ok, {absent} absent, {drifted} drifted, {conflict} conflict, {skipped} skipped"
    );

    if drifted + conflict > 0 {
        println!(
            "{}",
            "nothing drifted or conflicting is ever touched automatically — inspect, then decide"
                .dimmed()
        );
    }
}

pub fn would(entry: &Entry) {
    println!("  {} {}", "would".blue(), entry.what);
}

pub fn did(entry: &Entry) {
    println!("  {} {}", "done ".green(), entry.what);
}

pub fn refused(entry: &Entry, why: &str) {
    println!("  {} {} — {why}", "skip ".yellow(), entry.what);
}

pub fn mechanism_hint(plan: &Plan) {
    let pending: Vec<&str> = [Mechanism::Block, Mechanism::Wrapper]
        .iter()
        .filter(|m| {
            plan.entries
                .iter()
                .any(|e| e.mechanism == **m && e.state.needs_action())
        })
        .map(|m| m.label())
        .collect();

    if !pending.is_empty() {
        println!(
            "\n{} {} still to implement: {}",
            "note".dimmed(),
            pending.len(),
            pending.join(", ")
        );
    }
}

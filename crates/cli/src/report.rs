use std::collections::BTreeMap;
use std::io::{self, Write};

pub(crate) fn write(findings: &[kernel::Finding], out: &mut impl Write) -> io::Result<bool> {
    let mut ordered: Vec<&kernel::Finding> = findings.iter().collect();
    ordered.sort_by(order);
    let sheet = Report { ordered };
    sheet.emit(out)?;
    summary(sheet.ordered.len(), out)?;
    sheet.digest(out)?;
    Ok(!sheet.ordered.is_empty())
}

struct Report<'a> {
    ordered: Vec<&'a kernel::Finding>,
}

impl Report<'_> {
    fn emit(&self, out: &mut impl Write) -> io::Result<()> {
        for finding in &self.ordered {
            line(finding, out)?;
        }
        Ok(())
    }

    fn digest(&self, out: &mut impl Write) -> io::Result<()> {
        if self.ordered.is_empty() {
            return Ok(());
        }
        self.laws(out)?;
        self.hot(out)
    }

    fn laws(&self, out: &mut impl Write) -> io::Result<()> {
        let mut map: BTreeMap<&str, usize> = BTreeMap::new();
        for hit in &self.ordered {
            *map.entry(hit.law.as_str()).or_default() += 1;
        }
        write!(out, "by law:")?;
        for (law, hits) in map {
            write!(out, " {law}={hits}")?;
        }
        writeln!(out)
    }

    fn hot(&self, out: &mut impl Write) -> io::Result<()> {
        let mut map: BTreeMap<&str, usize> = BTreeMap::new();
        for hit in &self.ordered {
            *map.entry(hit.path.as_str()).or_default() += 1;
        }
        let mut list: Vec<(&str, usize)> = map.into_iter().collect();
        list.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(right.0)));
        write!(out, "hot files:")?;
        for (path, hits) in list.into_iter().take(5) {
            write!(out, " {path}={hits}")?;
        }
        writeln!(out)
    }
}

fn order(a: &&kernel::Finding, b: &&kernel::Finding) -> std::cmp::Ordering {
    (a.path.as_str(), a.line, a.col).cmp(&(b.path.as_str(), b.line, b.col))
}

fn line(finding: &kernel::Finding, out: &mut impl Write) -> io::Result<()> {
    writeln!(out, "{} {} {}", spot(finding), finding.law, finding.note)
}

fn spot(finding: &kernel::Finding) -> String {
    if finding.line == 0 {
        return finding.path.clone();
    }
    format!("{}:{}:{}", finding.path, finding.line, finding.col)
}

fn summary(errors: usize, out: &mut impl Write) -> io::Result<()> {
    if errors == 0 {
        return writeln!(out, "clean");
    }
    writeln!(out, "{}", plural(errors, "error"))
}

fn plural(count: usize, word: &str) -> String {
    if count == 1 {
        return format!("{count} {word}");
    }
    format!("{count} {word}s")
}

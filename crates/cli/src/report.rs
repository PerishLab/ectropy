use std::collections::BTreeMap;
use std::io::{self, Write};

pub(crate) fn write(
    findings: &[kernel::Finding],
    strict: bool,
    listed: bool,
    out: &mut impl Write,
) -> io::Result<bool> {
    let mut ordered: Vec<&kernel::Finding> = findings.iter().collect();
    ordered.sort_by(order);
    let sheet = Report { ordered };
    let faults = sheet.emit(kernel::Class::Fault, true, out)?;
    let blind = sheet.emit(kernel::Class::Blind, true, out)?;
    let debt = sheet.emit(kernel::Class::Debt, listed, out)?;
    summary(faults, blind, debt, out)?;
    sheet.digest(out)?;
    Ok(faults > 0 || (strict && blind > 0))
}

struct Report<'a> {
    ordered: Vec<&'a kernel::Finding>,
}

impl Report<'_> {
    fn emit(&self, class: kernel::Class, listed: bool, out: &mut impl Write) -> io::Result<usize> {
        let mut count = 0;
        for finding in &self.ordered {
            count += line(finding, class, listed, out)?;
        }
        Ok(count)
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

fn line(
    finding: &kernel::Finding,
    class: kernel::Class,
    listed: bool,
    out: &mut impl Write,
) -> io::Result<usize> {
    if finding.class != class {
        return Ok(0);
    }
    if listed {
        writeln!(
            out,
            "{} {} {}",
            spot(finding),
            label(class, finding),
            finding.note
        )?;
    }
    Ok(1)
}

fn spot(finding: &kernel::Finding) -> String {
    if finding.line == 0 {
        return finding.path.clone();
    }
    format!("{}:{}:{}", finding.path, finding.line, finding.col)
}

fn label(class: kernel::Class, finding: &kernel::Finding) -> &str {
    match class {
        kernel::Class::Blind => "blindspot",
        kernel::Class::Debt => "debt",
        kernel::Class::Fault => &finding.law,
    }
}

fn summary(faults: usize, blind: usize, debt: usize, out: &mut impl Write) -> io::Result<()> {
    if faults + blind + debt == 0 {
        return writeln!(out, "clean");
    }
    writeln!(
        out,
        "{}, {}, {}",
        plural(faults, "fault"),
        plural(blind, "blindspot"),
        plural(debt, "debt")
    )
}

fn plural(count: usize, word: &str) -> String {
    if count == 1 {
        return format!("{count} {word}");
    }
    format!("{count} {word}s")
}

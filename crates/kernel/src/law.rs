pub struct Law {
    pub name: &'static str,
    pub note: &'static str,
    pub exempt: bool,
}

pub const LAWS: &[Law] = &[
    Law {
        name: "ban",
        note: "banned syntax inside declared paths; narrow the ban itself",
        exempt: false,
    },
    Law {
        name: "block",
        note: "scope depth past the block limit",
        exempt: true,
    },
    Law {
        name: "burr",
        note: "a named underscore parameter accepted but unused",
        exempt: true,
    },
    Law {
        name: "combination",
        note: "a Boolean expression over more decision atoms than the limit",
        exempt: true,
    },
    Law {
        name: "comment",
        note: "a comment where policy disallows it",
        exempt: true,
    },
    Law {
        name: "coverage",
        note: "an unparsed region; a clean result cannot cover unread terrain",
        exempt: false,
    },
    Law {
        name: "dispatch",
        note: "adjacent equality branches repeating one subject",
        exempt: true,
    },
    Law {
        name: "fanout",
        note: "a scanned directory with more children than the limit",
        exempt: true,
    },
    Law {
        name: "file",
        note: "a source file longer than the file limit",
        exempt: true,
    },
    Law {
        name: "grant",
        note: "reserved syntax outside granted paths; widen the grant itself",
        exempt: false,
    },
    Law {
        name: "markup",
        note: "markup element depth past the markup limit",
        exempt: true,
    },
    Law {
        name: "param",
        note: "a function with more parameters than the limit",
        exempt: true,
    },
    Law {
        name: "path",
        note: "nesting past the path limit below the owning module root",
        exempt: true,
    },
    Law {
        name: "receiver",
        note: "a fourth free function sharing one receiver in a file",
        exempt: true,
    },
    Law {
        name: "shadow",
        note: "a literal densely copied bare from one source root",
        exempt: true,
    },
    Law {
        name: "word",
        note: "an unregistered compound declared name",
        exempt: true,
    },
];

pub fn find(name: &str) -> Option<&'static Law> {
    LAWS.iter().find(|law| law.name == name)
}

use gobble::*;

#[derive(Clone, PartialEq)]
pub enum Pass {
    Front,
    GTemplate,
    Markdown,
    Table,
    Exec(String),
}

pub struct Section<'a> {
    passes: Vec<Pass>,
    data: &'a str,
}

parser! {(PassItem->Pass)
    or!(
        "front".asv(Pass::Front),
        "f".asv(Pass::Front),
        "go".asv(Pass::GTemplate),
        "gtmpl".asv(Pass::GTemplate),
        "g".asv(Pass::GTemplate),
        "markdown".asv(Pass::Markdown),
        "md".asv(Pass::Markdown),
        "table".asv(Pass::Table),
        ("exec ",Any.except("\n|").plus()).map(|(_,v)|Pass::Exec(v)),
    )
}

parser! {(PassLine->Vec<Pass>)
    (">---",star(last(ws__(.plus(),PassItem))
}

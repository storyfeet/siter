use gobble::*;

#[derive(Clone, PartialEq, Debug)]
pub enum Pass {
    Front,
    GTemplate,
    Markdown,
    Table,
    Exec(String),
}

#[derive(Clone, PartialEq, Debug)]
pub struct Section<'a> {
    passes: Vec<Pass>,
    data: &'a str,
}

pub fn section_pull<'a>(s: &'a str) -> SectionPull<'a> {
    SectionPull {
        p: SectionPos.pull(s),
    }
}

pub struct SectionPull<'a> {
    p: gobble::pull::PullParser<'a, SectionPos, EOI>,
}

impl<'a> Iterator for SectionPull<'a> {
    type Item = Result<Section<'a>, StrError<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        self.p.next().map(|r| {
            r.map(|(passes, dt)| Section {
                passes,
                data: dt.on_str(self.p.s),
            })
        })
    }
}

parser! {(SectionPos->(Vec<Pass>,StrPos))
    (maybe(PassLine),SecData).map(|(p_op,dt)|(p_op.unwrap_or(Vec::new()),dt))
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
    last(">---",sep_until_ig(ws__(PassItem),"|","\n"))
}

parser! {(SecData->Pos<()>)
    pos_ig(repeat_until((Any.except("\n").star(),maybe("\n")),peek(or(eoi.ig(),PassLine.ig()))))
}

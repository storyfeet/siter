use super::pass::{Pass, Section};
use gobble::*;

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
                s: dt.on_str(self.p.s),
            })
        })
    }
}

parser! {(SectionPos->(Vec<Pass>,StrPos))
    (maybe(PassLine),SecData).map(|(p_op,dt)|(p_op.unwrap_or(Vec::new()),dt))
}

parser! {(Params->String)
    ws_(Any.except("\n|").plus()).map(|v|v.trim().to_string())
}

parser! {(PassItem->Pass)
    or!(
        "toml".asv(Pass::Toml),
        "go".asv(Pass::GTemplate),
        "gtmpl".asv(Pass::GTemplate),
        "g".asv(Pass::GTemplate),
        "markdown".asv(Pass::Markdown),
        "md".asv(Pass::Markdown),
        "#".asv(Pass::Comment),
        keyword("files").map(|_|Pass::Files),
        (keyword("dirs"),Params).map(|(_,v)|Pass::SetDirs(v)),
        (keyword(or("template","tp")),Params).map(|(_,v)|Pass::Template(v)),
        (keyword("set"),Params).map(|(_,v)| Pass::Set(v)),
        (keyword("table"),Params).map(|(_,v)|Pass::Table(v)),
        (keyword("exec"),Params).map(|(_,v)|Pass::Exec(v)),
    )
}

parser! {(PassLine->Vec<Pass>)
    last(">---",sep_until_ig(ws__(PassItem),"|","\n"))
}

parser! {(SecData->Pos<()>)
    pos_ig(repeat_until((Any.except("\n").star(),maybe("\n")),peek(or(eoi.ig(),PassLine.ig()))))
}

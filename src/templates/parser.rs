use super::pass::{FSource, FileEntry, Pass, Section};
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

parser! {(PFileEntry->FileEntry)
    (maybe(first(ws_(common::Ident),'=')),Params).map(|(var,path)|FileEntry{var,path})
}

parser! {(SectionPos->(Vec<Pass>,StrPos))
    (PassLine,SecData).map(|(p,dt)|(p,dt))
}

parser! {(Source->FSource)
    or!(
        or("tp","templates").asv(FSource::Templates),
        or("ct","content").asv(FSource::Content),
        or("st","static").asv(FSource::Static),
        "".asv(FSource::Content),
    )
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
        (keyword("files"),ws_(Source)).map(|(_,s)|Pass::Files(s)),
        (keyword("dirs"),ws_(Source)).map(|(_,s)|Pass::Dirs(s)),
        (keyword(or("template","tp")),maybe(Params)).map(|(_,v)|Pass::Template(v)),
        (keyword("set"),Params).map(|(_,v)| Pass::Set(v)),
        (keyword("table"),Params).map(|(_,v)|Pass::Table(v)),
        (keyword("exec"),Params).map(|(_,v)|Pass::Exec(v)),
    )
}

parser! {(PassItems->Vec<Pass>)
    sep_until_ig(ws__(PassItem),"|",or_ig!("\n",eoi))
}
parser! {(PassLine->Vec<Pass>)
    or(
        last(">---",PassItems),
        peek(not(">").one()).map(|_|Vec::new())
    )
}

parser! {(SecData->Pos<()>)
    pos_ig(repeat_until((Any.except("\n").star(),maybe("\n")),peek(or_ig!(eoi,">---"))))
}

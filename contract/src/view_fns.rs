pub use crate::*;

#[near_bindgen]
impl Contract {
    pub fn view_papers(&self) -> Vec<TokenId>{
        let mut papers = Vec::new();
        for paperid in self.papersmetadata.keys(){
            papers.push(paperid);
        }
        papers
    }
    pub fn view_paper_meta(&self,token_id: &TokenId) -> PaperMetadata{
        self.papersmetadata.get(token_id).unwrap()
    }
}

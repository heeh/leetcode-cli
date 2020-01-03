//! mod for parse resp data
use crate::err::Error;
use super::models::*;
use serde_json::Value;

/// problem parser
pub fn problem(problems: &mut Vec<Problem>, v: Value) -> Result<(), Error> {
    let pairs = v.get("stat_status_pairs")?.as_array()?;
    for p in pairs {
        let stat = p.get("stat")?.as_object()?;
        let total_acs = stat.get("total_acs")?.as_f64()? as f32;
        let total_submitted = stat.get("total_submitted")?.as_f64()? as f32;
        
        problems.push(Problem{
            category: v.get("category_slug")?.as_str()?.to_string(),
            fid: stat.get("frontend_question_id")?.as_i64()? as i32,
            id: stat.get("question_id")?.as_i64()? as i32,
            level: p.get("difficulty")?.as_object()?.get("level")?.as_i64()? as i32,
            locked: p.get("paid_only")?.as_bool()?,
            name: stat.get("question__title")?.as_str()?.to_string(),
            percent: total_acs / total_submitted * 100.0,
            slug: stat.get("question__title_slug")?.as_str()?.to_string(),
            starred: p.get("is_favor")?.as_bool()?,
            status: p.get("status")?.as_str().unwrap_or("Null").to_string(),
            desc: String::new(),
        });
    }

    return Ok(());
}

/// desc parser
pub fn desc(q: &mut Question, v: Value) -> Result<(), Error> {
    let o = &v.as_object()?.get("data")?.as_object()?.get("question")?.as_object()?;

    *q = Question {
        content: o.get("content")?.as_str().unwrap_or("").to_string(),
        stats: serde_json::from_str(o.get("stats")?.as_str()?)?,
        defs: serde_json::from_str(o.get("codeDefinition")?.as_str()?)?,
        case: o.get("sampleTestCase")?.as_str()?.to_string(),
        metadata: serde_json::from_str(o.get("metaData")?.as_str()?)?,
        test: o.get("enableRunCode")?.as_bool()?,
        t_content: o.get("translatedContent")?.as_str().unwrap_or("").to_string(),
    };

    Ok(())
}
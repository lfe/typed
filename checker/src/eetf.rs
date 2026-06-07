use crate::sexp::types::*;
use std::io::Write;

const VERSION_TAG: u8 = 131;
const SMALL_INTEGER_TAG: u8 = 97;
const INTEGER_TAG: u8 = 98;
const ATOM_UTF8_TAG: u8 = 118;
const SMALL_TUPLE_TAG: u8 = 104;
const NIL_TAG: u8 = 106;
const LIST_TAG: u8 = 108;
const BINARY_TAG: u8 = 109;

pub fn encode_forms(forms: &[(SExp, usize)]) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.push(VERSION_TAG);
    encode_form_list(forms, &mut buf);
    buf
}

fn encode_form_list(forms: &[(SExp, usize)], buf: &mut Vec<u8>) {
    if forms.is_empty() {
        buf.push(NIL_TAG);
        return;
    }
    buf.push(LIST_TAG);
    let len = forms.len() as u32;
    buf.extend_from_slice(&len.to_be_bytes());
    for (form, line) in forms {
        buf.push(SMALL_TUPLE_TAG);
        buf.push(2);
        encode_sexp(form, buf);
        encode_integer(*line as i64, buf);
    }
    buf.push(NIL_TAG);
}

fn encode_sexp(sexp: &SExp, buf: &mut Vec<u8>) {
    match sexp {
        SExp::Symbol(s) => encode_atom(&s.value, buf),
        SExp::Keyword(k) => encode_atom(&k.name, buf),
        SExp::String(s) => encode_binary(s.value.as_bytes(), buf),
        SExp::Number(n) => {
            if let Ok(v) = n.value.parse::<i64>() {
                encode_integer(v, buf);
            } else {
                encode_binary(n.value.as_bytes(), buf);
            }
        }
        SExp::Nil(_) => {
            buf.push(NIL_TAG);
        }
        SExp::List(l) => {
            if l.elements.is_empty() {
                buf.push(NIL_TAG);
                return;
            }
            buf.push(LIST_TAG);
            let len = l.elements.len() as u32;
            buf.extend_from_slice(&len.to_be_bytes());
            for elem in &l.elements {
                encode_sexp(elem, buf);
            }
            buf.push(NIL_TAG);
        }
    }
}

fn encode_atom(name: &str, buf: &mut Vec<u8>) {
    let bytes = name.as_bytes();
    buf.push(ATOM_UTF8_TAG);
    let len = bytes.len() as u16;
    buf.extend_from_slice(&len.to_be_bytes());
    buf.extend_from_slice(bytes);
}

fn encode_integer(val: i64, buf: &mut Vec<u8>) {
    if (0..=255).contains(&val) {
        buf.push(SMALL_INTEGER_TAG);
        buf.push(val as u8);
    } else {
        buf.push(INTEGER_TAG);
        buf.extend_from_slice(&(val as i32).to_be_bytes());
    }
}

fn encode_binary(data: &[u8], buf: &mut Vec<u8>) {
    buf.push(BINARY_TAG);
    let len = data.len() as u32;
    buf.extend_from_slice(&len.to_be_bytes());
    buf.extend_from_slice(data);
}

pub fn write_eetf_file(path: &str, data: &[u8]) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;
    file.write_all(data)?;
    Ok(())
}

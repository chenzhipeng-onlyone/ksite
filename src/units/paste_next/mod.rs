#![allow(unused)]

/*
剪贴板和文件存储服务，读写本地文件，hash分流

账户登录
文件存储
加密
原生js糊的轻量的界面，借鉴一点点react之类的东西
在用户之间分享
类似 git fork, cow
尽量优化性能
有限制的自增和其他结合的 ID

区分创建与插入？评估性能影响
密码用hash，用户名和密码都固定宽度？优化性能？
页面缓存，LRU？

https://www.runoob.com/sqlite/sqlite-intro.html

密码hash加盐

用户频率限制，空间限制，会员制？
写入原始内容，前段自行处理转义
内部用数字存储cid，文件名也是数字

/paste/raw/:id

每个账户1024条，固定大小的“地址空间”？
不要超过i64
不要做服务端解密

如果加密，那么mime就失去了意义，所以用mime来存储是否加密的信息

-----
/ksite
/ksite.db

*/
use crate::{db, strip_str};
// use crate::utils::slot;
use axum::extract::{Form, Path};
use axum::response::{Html, Redirect};
use axum::routing::MethodRouter;
use axum::Router;
use serde::Deserialize;

fn db_init() {
    // uid: user id (hashed)
    // upw: user password (hashed)
    // cid: clipboard id (i64, but > 0)
    // cpw: clipboard password (hashed) (may be NULL)
    // desc: clipborad description
    // mime: use this as the content-type, and is encrypted flag
    let sqls = [
        strip_str! {"CREATE TABLE paste_user (
            uid BLOB PRIMARY KEY,
            upw BLOB
        )"},
        strip_str! {"CREATE TABLE paste_data (
            cid INTEGER PRIMARY KEY AUTOINCREMENT,
            uid BLOB,
            desc BLOB,
            mime BLOB
        )"},
    ];
    for sql in sqls {
        db!(sql).ok();
    }
}
fn db_user_cu(uid: &[u8], upw: &[u8]) {
    db!("REPLACE INTO paste_user VALUES (?1, ?2)", [uid, upw]).unwrap();
}
fn db_user_r(uid: &[u8]) -> Option<(Vec<u8>,)> {
    db!("SELECT upw FROM paste_user WHERE uid = ?", [uid], ^(0)).ok()
}
fn db_user_d(uid: &[u8]) {
    db!("DELETE FROM paste_user WHERE uid = ?", [uid]).unwrap();
}
fn db_data_c(uid: &[u8], desc: &[u8], mime: &[u8]) -> i64 {
    let sql = "INSERT INTO paste_data VALUES (NULL, ?1, ?2, ?3)";
    db!(sql, [uid, desc, mime], &).unwrap()
}
fn db_data_u_desc(cid: i64, desc: &[u8]) -> bool {
    let sql = "UPDATE paste_data SET desc = ?2 WHERE cid = ?1";
    db!(sql, [cid, desc]).is_ok()
}
fn db_data_u_mime(cid: i64, mime: &[u8]) -> bool {
    let sql = "UPDATE paste_data SET mime = ?2 WHERE cid = ?1";
    db!(sql, [cid, mime]).is_ok()
}
fn db_data_r(cid: i64) -> Option<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    let sql = "SELECT * FROM paste_data WHERE cid = ?";
    db!(sql, [cid], ^(1, 2, 3)).ok()
}
fn db_data_r_by_user(uid: i64) -> Vec<(i64, Vec<u8>, Vec<u8>)> {
    let sql = "SELECT * FROM paste_data WHERE uid = ?";
    db!(sql, [uid], (0, 2, 3)).unwrap()
}
fn db_data_d(cid: i64) -> bool {
    db!("DELETE FROM paste_data WHERE cid = ?", [cid]).is_ok()
}

const CID_CHARS: [u8; 36] = *b"0123456789abcdefghijklmnopqrstuvwxyz";
const CID_MAX_LEN: usize = 16; // javascript: (2**64).toString(36).length == 13

/// Convert CID integer to string.
fn int2str(i: i64, buf: &mut [u8; CID_MAX_LEN]) -> &[u8] {
    const L: usize = CID_CHARS.len();
    let mut i = i as usize;
    let mut p = CID_MAX_LEN - 1;
    while i != 0 {
        unsafe { *buf.get_unchecked_mut(p) = *CID_CHARS.get_unchecked(i % L) };
        p -= 1;
        i /= L;
    }
    &buf[p + 1..]
}

/// Convert CID string to integer.
fn str2int(s: &[u8]) -> i64 {
    const L: i64 = CID_CHARS.len() as _;
    let mut ret = 0;
    for c in s {
        let c = match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'z' => c - b'a' + 10,
            _ => unreachable!(),
        } as i64;
        ret = ret * L + c;
    }
    ret
}

#[allow(unused)]
fn test_int2str2int() {
    const SRC: i64 = 123454323;
    let mut cid_buf = [0; CID_MAX_LEN];
    let cid = int2str(SRC, &mut cid_buf);
    assert_eq!(str2int(cid), SRC);
}

pub fn service() -> Router {
    // db_init();
    // dbg!(db!("VACUUM"));
    // mentions about the path later?
    Router::new()
}

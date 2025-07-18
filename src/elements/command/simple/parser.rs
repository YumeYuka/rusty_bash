//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder, utils};
use super::SimpleCommand;
use crate::elements::command;
use crate::elements::substitution::Substitution;
use crate::elements::word::{Word, WordMode};
use crate::error::parse::ParseError;

impl SimpleCommand {
    fn eat_substitution(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> Result<bool, ParseError> {
        let read_var = core.substitution_builtins.contains_key(&ans.command_name);

        if let Some(s) = Substitution::parse(feeder, core, read_var)? {
            ans.text += &s.text;

            if core.substitution_builtins.contains_key(&ans.command_name) 
            || ans.command_name == "eval" {
                ans.substitutions_as_args.push(s);
            }else{
                ans.substitutions.push(s);
            }
            Ok(true)
        }else{
            Ok(false)
        }
    }

    fn eat_word(feeder: &mut Feeder, ans: &mut SimpleCommand, core: &mut ShellCore)
        -> Result<bool, ParseError> {
        let w = match Word::parse(feeder, core, None) {
            Ok(Some(w)) => w,
            Err(e) => {
                feeder.rewind();
                return Err(e);
            },
            _ => return Ok(false),
        };

        if ans.words.is_empty() {
            ans.lineno = feeder.lineno;
            if utils::reserved(&w.text) {
                return Ok(false);
            }

            ans.command_name = w.text.clone();
        }

        if ans.words.is_empty() || ans.continue_alias_check {
            if ans.set_alias(&w, core, feeder)? {
                return Ok(true);
            }
        }

        ans.text += &w.text;
        ans.words.push(w);

        Ok(true)
    }

    fn set_alias(&mut self, word: &Word,
                 core: &mut ShellCore, feeder: &mut Feeder) -> Result<bool, ParseError> {
        self.continue_alias_check = false;
        let mut w = word.text.clone();
        if ! core.replace_alias(&mut w) {
            return Ok(false);
        }

        self.continue_alias_check = w.ends_with(" ");
        let mut feeder_local = Feeder::new(&mut w);

        while Self::eat_substitution(&mut feeder_local, self, core)? {
            command::eat_blank_with_comment(&mut feeder_local, core, &mut self.text);
        }

        loop {
            match Word::parse(&mut feeder_local, core, Some(WordMode::Alias)) {
                Ok(Some(w)) => {
                    if w.text.starts_with("#") && self.words.is_empty() {
                        break;
                    }
                    self.text.push_str(&w.text);
                    self.words.push(w);
                },
                _    => break,
            }
            command::eat_blank_with_comment(&mut feeder_local, core, &mut self.text);
        }

        if let Some(lst) = self.words.last() {
            if lst.text == "\\" {
                self.words.pop();
                feeder_local.replace(0, "\\");
            }
        }

        feeder.replace(0, &feeder_local.consume(feeder_local.len()));

        if self.words.is_empty() && self.substitutions.is_empty() {
            self.invalid_alias = true;
            return Ok(false);
        }

        Ok(true)
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        let mut ans = Self::default();
        feeder.set_backup();

        while command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text)?
        || Self::eat_substitution(feeder, &mut ans, core)? {
            command::eat_blank_with_comment(feeder, core, &mut ans.text);
        }

        loop {
            command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text)?;

            if core.substitution_builtins.contains_key(&ans.command_name) 
            || ans.command_name == "eval" {
                if Self::eat_substitution(feeder, &mut ans, core)? {
                    continue;
                }
            }

            command::eat_blank_with_comment(feeder, core, &mut ans.text);
            if ! Self::eat_word(feeder, &mut ans, core)? {
                break;
            }
        }

        if ans.invalid_alias {
            feeder.pop_backup();
            feeder.consume(feeder.len());
            return Ok(None);
        }

        if ans.substitutions.len() + ans.words.len() + ans.redirects.len() > 0 {
            feeder.pop_backup();
            Ok(Some(ans))
        }else{
            feeder.rewind();
            Ok(None)
        }
    }
}

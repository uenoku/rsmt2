//! Activation literal type and helpers.
//!
//! For an explanation of what activation literal are, see
//! [the discussion below][why actlits].
//!
//! **NB**: while `rmst2`'s actlit API declares some constant symbols in the
//! underlying solver, these will not appear in the result of
//! [`get_model`](get_model) queries.
//!
//!
//! # Relevant functions on solvers
//!
//! - [`get_actlit`](../trait.Solver.html#method.get_actlit)
//! - [`get_sactlit`](../trait.Solver.html#method.get_sactlit)
//! - [`de_actlit`](../trait.Solver.html#method.de_actlit)
//! - [`set_actlit`](../trait.Solver.html#method.set_actlit)
//! - [`assert_act`](../trait.Solver.html#method.assert_act)
//! - [`assert_act_with`](../trait.Solver.html#method.assert_act_with)
//! - [`print_check_sat_act`](../trait.Solver.html#method.print_check_sat_act)
//! - [`check_sat_act`](../trait.Solver.html#method.check_sat_act)
//! - [`check_sat_act_or_unk`]
//!   (../trait.Solver.html#method.check_sat_act_or_unk)
//!
//!
//!
//! # Usage
//!
//! First, one can of course create activation literals by hand, and use them
//! in `check-sat`s with [`check_sat_assuming`][check sat ass]:
//!
//! ```
//! use rsmt2::* ;
//!
//! let mut solver = Solver::default(()).unwrap() ;
//! solver.declare_const("x", "Int").unwrap() ;
//!
//! solver.declare_const("actlit", "Bool").unwrap() ;
//! solver.assert("\
//!   (=> actlit \
//!       (and (> x 0) (< x 3) (= (mod x 3) 0))\
//!   )\
//! ").unwrap() ;
//! assert!{
//!   ! solver.check_sat_assuming( Some("actlit") ).unwrap()
//! }
//! solver.assert("(not actlit)").unwrap() ;
//!
//! solver.declare_const("other_actlit", "Bool").unwrap() ;
//! solver.assert("\
//!   (=> other_actlit \
//!       (and (> x 7) (= (mod x 2) 0))\
//!   )\
//! ").unwrap() ;
//! assert!{
//!   solver.check_sat_assuming( Some("other_actlit") ).unwrap()
//! }
//! solver.assert("(not other_actlit)").unwrap() ;
//!
//! solver.kill().unwrap()
//! ```
//!
//! The activation literal API makes this process more straightforward:
//!
//! ```
//! use rsmt2::* ;
//!
//! let mut solver = match Solver::default(()) {
//!   Ok(kid) => kid,
//!   Err(e) => panic!("Could not spawn solver kid: {:?}", e)
//! } ; solver.path_tee("log.smt2").unwrap() ;
//!
//! solver.declare_const("x", "Int").unwrap() ;
//!
//! let actlit = solver.get_actlit().unwrap() ;
//! solver.assert_act(& actlit, "(> x 0)").unwrap() ;
//! solver.assert_act(& actlit, "(< x 3)").unwrap() ;
//! solver.assert_act(& actlit, "(= (mod x 3) 0)").unwrap() ;
//!
//! assert!{
//!   ! solver.check_sat_act( Some(& actlit) ).unwrap()
//! }
//! solver.de_actlit(actlit).unwrap() ;
//! // At this point `actlit` has been consumed. So it's a bit safer than the
//! // version above, since use-after-deactivate is not possible.
//!
//! let actlit = solver.get_actlit().unwrap() ;
//! solver.assert_act(& actlit, "(> x 7)").unwrap() ;
//! solver.assert_act(& actlit, "(= (mod x 2) 0)").unwrap() ;
//! assert!{
//!   solver.check_sat_act( Some(& actlit) ).unwrap()
//! }
//! solver.de_actlit(actlit).unwrap() ;
//!
//! solver.kill().unwrap()
//! ```
//!
//!
//! **NB**: under the hood, `rmst2` declares a constant boolean symbol for each
//! actlit. Hence, there is a (very low) risk of collision with the user's
//! symbol. The internal actlits are named `"|rsmt2 actlit <uid>|"`. Any symbol
//! starting with `"|rsmt2 actlit "` is assumed to be a `rsmt2` actlit. In
//! particular, such symbols will be pruned out of `get_model` queries (if at
//! least one actlit was requested since the last reset).
//!
//! ```
//! use rsmt2::* ;
//! use rsmt2::parse::* ;
//!
//! struct Parser ;
//! impl<'a, 'b> IdentParser<String, String, & 'a str> for & 'b Parser {
//!   fn parse_ident(self, s: & 'a str) -> SmtRes<String> {
//!     Ok(s.to_string())
//!   }
//!   fn parse_type(self, s: & 'a str) -> SmtRes<String> {
//!     Ok(s.to_string())
//!   }
//! }
//! impl<'a, 'b> ModelParser<
//!   String, String, String, & 'a str
//! > for & 'b Parser {
//!   fn parse_value(
//!     self, s: & 'a str,
//!     _: & String, _: & Vec<(String, String)>, _: & String
//!   ) -> SmtRes<String> {
//!     Ok(s.to_string())
//!   }
//! }
//!
//! let mut solver = match Solver::default(& Parser) {
//!   Ok(kid) => kid,
//!   Err(e) => panic!("Could not spawn solver kid: {:?}", e)
//! } ;
//!
//! solver.declare_const("x", "Int").unwrap() ;
//!
//! let actlit = solver.get_actlit().unwrap() ;
//! let mut buf: Vec<u8> = vec![] ;
//! actlit.write(& mut buf).unwrap() ;
//! assert_eq!{
//!   "|rsmt2 actlit 0|",
//!   ::std::str::from_utf8(& buf).unwrap()
//! }
//!
//! solver.assert_act(& actlit, "(> x 7)").unwrap() ;
//! solver.assert_act(& actlit, "(= (mod x 2) 0)").unwrap() ;
//! assert!{
//!   solver.check_sat_act( Some(& actlit) ).unwrap()
//! }
//!
//! let model = solver.get_model_const().unwrap() ;
//! let mut model = model.into_iter() ;
//! if let Some((x, int, n)) = model.next() {
//!   assert_eq!{ x, "x" }
//!   assert_eq!{ int, "Int" }
//!   use std::str::FromStr ;
//!   let n = i64::from_str(& n).unwrap() ;
//!   println!("{}", n) ;
//!   assert!{ n > 7 }
//!   assert!{ n % 2 == 0 }
//! } else {
//!   panic!("expected the model for `x`")
//! }
//! assert_eq!{
//!   model.next(), None
//! }
//!
//! solver.de_actlit(actlit).unwrap() ;
//!
//! solver.kill().unwrap()
//! ```
//!
//!
//!
//! # Discussion on activation literals
//!
//! The activation literal technique is a much more efficient alternative to
//! the `push`/`pop` SMT-LIB commands. When a `pop` command is issued, solvers
//! usually reset themselves and re-declare/assert whatever was before the last
//! push.
//!
//! Activation literals are boolean nullary symbols controlling the activation
//! of some assertions.
//!
//! For instance
//!
//! ```smt2
//! (declare-fun x () Int)
//!
//! (push 1)
//!   (assert (> x 0))
//!   (assert (< x 3))
//!   (assert (= (mod x 3) 0))
//!   (check-sat)
//!   ; unsat
//! (pop 1)
//!
//! (push 1)
//!   (assert (> x 7))
//!   (assert (= (mod x 2) 0))
//!   (check-sat)
//!   (get-value (x))
//! (pop 1)
//! ```
//!
//! can be encoded with activation literals as
//!
//! ```smt
//! (declare-fun x () Int)
//!
//! (declare-fun actlit_1 () Bool)
//! (declare-fun actlit_2 () Bool)
//!
//! (assert (=> actlit_1 (> x 0)) )
//! (assert (=> actlit_2 (< x 3)) )
//! (assert (=> actlit_2 (= (mod x 3) 0)) )
//! (check-sat actlit_1 actlit_2) ; <--- Conditional check-sat
//!                               ;      usually called "check-sat-assuming"
//! ; unsat
//!
//! (assert (not actlit_2)) ; <--- Actlit deactivation
//!                         ;      all its assertions basically disappear
//!
//! (declare-fun actlit_3 () Bool)
//!
//! (assert (=> actlit_3 (> x 7)) )
//! (assert (=> actlit_3 (= (mod x 2) 0)) )
//! (check-sat actlit_1 actlit_3)
//! ; sat
//! (get-value (x))
//! ```
//!
//! This is much more efficient than `push`/`pop`: the conditional `check-sat`s
//! basically force the activation literals directly in the SAT part of the SMT
//! solver. Long story short, this means everything the solver learns during
//! the checks is still valid afterwards. Conversely, after a `pop` solvers are
//! usually unable to decide what to keep from the checks before the `pop`, and
//! thus drop everything.
//!
//! Actlits are **not** equivalent to `push`/`pop` however. Pushing a scope
//! allows to declare/define function symbols and then discard them, while
//! keeping whatever's outside of the scope. Actlits (mostly) just guard
//! assertions and cannot accomplish this.
//!
//! [why actlits]: #discussion-on-activation-literals
//! (Activation literals, why?)
//! [SActlit]: struct.SActlit.html (SActlit documentation)
//! [Actlit]: struct.Actlit.html (Actlit documentation)
//! [phantom]: https://doc.rust-lang.org/std/marker/struct.PhantomData.html
//! (PhantomData documentation)
//! [check sat ass]: ../trait.Solver.html#method.check_sat_assuming
//! (check_sat_assuming function)
//! [get_model]: ../trait.Solver.html#method.get_model (get_model function)

pub use solver::Actlit ;
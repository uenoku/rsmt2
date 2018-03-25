//! A few examples of using rsmt2.

/// Convenience macro.
#[cfg(test)]
macro_rules! smtry {
  ($e:expr, failwith $( $msg:expr ),+) => (
    match $e {
      Ok(something) => something,
      Err(e) => panic!( $($msg),+ , e)
    }
  ) ;
}

pub mod simple ;
pub mod print_time ;

#[cfg(test)]
fn get_solver<Parser>(p: Parser) -> ::Solver<Parser> {
  match ::Solver::default(p) {
    Ok(solver) => solver,
    Err(e) => panic!("Could not spawn solver solver: {:?}", e)
  }
}
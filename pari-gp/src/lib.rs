//! # PARI/GP
//! 
//! [PARI/GP] is a cross platform and open-source computer algebra system designed for fast
//! computations in number theory.
//! 
//! This library contains a macro that calls `gp` internally and translates its output back into
//! rust code, as a way to be able to write gp code inside rust code.
//! 
//! ```
//! use pari_gp::gp;
//! 
//! let ten_factorial: u64 = gp!{
//!     (4 + 6)!
//! };
//! 
//! // you can import rust variables into the code (but they cannot be altered,
//! // think of them as a const inside the gp block)
//! let expression: u64 = gp!{
//!     n = @ten_factorial;
//!     
//!     n^2
//! };
//! 
//! assert_eq!(expression, ten_factorial * ten_factorial);
//! 
//! // all the PARI/GP functionalities are here
//! let precision = 9;
//! let ramanujan_delta: Vec<i32> = gp!{
//!     mfcoefs(mfDelta(), @precision)
//! };
//! 
//! assert_eq!(
//!     ramanujan_delta,
//!     vec![0, 1, -24, 252, -1472, 4830, -6048, -16744, 84480, -113643]
//! );
//! ```
//! 
//! ## Warning
//! 
//! <div class="warning">
//! This project is extremely new, so it's missing a lot of features and I'm confident it has
//! lots of bugs I haven't noticed yet.
//! </div>
//! 
//! ## Usage
//! 
//! This library is merely converting the stuff inside the `gp!{}` macro into a string, then
//! feeding that string into a `gp` subcommand. Therefore, you **must have PARI/GP** installed
//! on your machine. Not only that, but you also need to **include the path** to the installed
//! `gp.exe` as a PATH variable.
//! 
//! For Windows users, you may verify that you have it as a path variable by opening the
//! command line or powershell anywhere and running `gp.exe` (running `gp` in the powershell can
//! conflict with the `Get-ItemProperty` cmdlet). If that starts up GP/PARI, then you should
//! be good to go.
//! 
//! The translation from Rust to GP and vice-versa is done through two traits: [`FromGp`] and
//! [`IntoGp`]. Most primitives (like numbers and vectors) already implement both traits. Whatever
//! is captured in the `stdout` output of the command call is what gets parsed into the output.
//! Therefore, I suggest writing the expression you wish to output at the end of the macro.
//! 
//! ```no_run
//! # type T = Vec<u16>;
//! # type S = Vec<u16>;
//! # use pari_gp::gp;
//! // this piece of code would work if T: IntoGp and S: FromGp
//! 
//! let input: T = T::default();
//! 
//! let output: S = gp!{
//!     some_operation(@input)
//! };
//! ```
//! 
//! Note that the way Rust handles procedural macros makes it so I have no information about the
//! new lines inside the macro. Thus, you should **always end lines with** `;` as you see fit.
//! Unless of course, you want to break down a function call into multiple lines, such as in a
//! for loop.
//! 
//! ```
//! # use pari_gp::gp;
//! let sum: u64 = gp!{
//!     ans = 0;
//!     
//!     for(i = 1, 10,
//!         ans += i^2
//!     );
//! 
//!     ans
//! };
//! ```
//! 
//! [PARI/GP]: (https://pari.math.u-bordeaux.fr/)

pub use gp_macro::gp;
pub use gp_runtime::{FromGp, IntoGp, GpError, ParseError};

#[doc(hidden)]
pub mod __runtime {
    pub use gp_runtime::{run, try_from_gp, FromGp, IntoGp};
}
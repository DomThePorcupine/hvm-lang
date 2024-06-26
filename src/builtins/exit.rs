use super::util::FunctionLikeHosted;
use crate::builtins::util::AsDefFunction;
use hvmc::{
  host::Host,
  run::{Def, LabSet, Mode, Net, Port, Tag, Wire},
  stdlib::HostedDef,
};
use parking_lot::Mutex;
use std::sync::Arc;

pub(crate) fn add_exit_def(host: Arc<Mutex<Host>>) {
  /// `HVM.exit`.
  /// Implements the following reduction rule
  ///
  /// ```txt
  /// ExitDef ~ (a b)
  /// ---
  /// ExitDefGetStatus ~ a & <dangling> ~ b
  /// ```
  struct ExitDef;

  impl AsDefFunction for ExitDef {
    fn call<M: Mode>(&self, net: &mut Net<M>, input: Wire, output: Wire) {
      // Purposefully deadlock "output" to prevent further reduction
      drop(output);

      if M::LAZY {
        net.normal_from(input.clone());
        exit(net, input.load_target());
      } else {
        // The manual deref converts the type of (exit, exit) into Dynamic
        #[allow(clippy::explicit_auto_deref)]
        net.link_wire_port(input, Port::new_ref(&*Def::new(LabSet::ALL, (exit, exit))));
      }

      fn exit<M: Mode>(_: &mut Net<M>, port: Port) {
        match port.tag() {
          Tag::Num => {
            std::process::exit(port.num().try_into().unwrap_or(-1));
          }
          _ => {
            std::process::exit(-1);
          }
        }
      }
    }
  }

  host
    .lock()
    .insert_def("HVM.exit", unsafe { HostedDef::new_hosted(LabSet::ALL, FunctionLikeHosted(ExitDef)) });
}

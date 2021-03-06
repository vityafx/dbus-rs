/* This example creates a D-Bus server with the following functionality:
   It registers the "com.example.dbustest" name, creates a "/hello" object path,
   which has an "com.example.dbustest" interface.

   The interface has a "Hello" method (which takes no arguments and returns a string),
   and a "HelloHappened" signal (with a string argument) which is sent every time
   someone calls the "Hello" method.
*/


extern crate dbus;
extern crate futures;
extern crate tokio_timer;
extern crate dbus_tokio;

use std::time::Duration;
use std::sync::Arc;
use dbus::{Connection, BusType, NameFlag};
use dbus::tree::MethodErr;
use dbus_tokio::tree::{AFactory, ATree};
use tokio_timer::*;

use futures::Future;

fn main() {
    // Let's start by starting up a connection to the session bus and register a name.
    let c = Connection::get_private(BusType::Session).unwrap();
    // TODO: Should we have a TokioConnection???
    c.register_name("com.example.dbustest", NameFlag::ReplaceExisting as u32).unwrap();

    // The choice of factory tells us what type of tree we want,
    // and if we want any extra data inside. We pick the simplest variant.
    let f = AFactory::new_afn::<()>();

    // We create the signal first, since we'll need it in both inside the method callback
    // and when creating the tree.
    let signal = Arc::new(f.signal("HelloHappened", ()).sarg::<&str,_>("sender"));
    let signal2 = signal.clone();

    // We create a tree with one object path inside and make that path introspectable.
    let tree = f.tree(ATree::new()).add(f.object_path("/hello", ()).introspectable().add(

        // We add an interface to the object path...
        f.interface("com.example.dbustest", ()).add_m(

            // ...and a method inside the interface.
            f.amethod("Hello", (), move |m| {

                // This is the callback that will be called when another peer on the bus calls our method.
                // the callback receives "MethodInfo" struct and can return either an error, or a list of
                // messages to send back.
                let timer = Timer::default();
                let sleep_future = timer.sleep(Duration::from_millis(500));

                // These are the variables we need after the timeout period. We need to 
                // clone all strings now, because the tree might get destroyed during the sleep.
                let sender = m.msg.sender().unwrap().into_static();
                let (pname, iname) = (m.path.get_name().clone(), m.iface.get_name().clone());
                let mret = m.msg.method_return();
                let signal3 = signal.clone();

                sleep_future.and_then(move |_| {
                    let s = format!("Hello {}!", sender);
                    let mret = mret.append1(s);
                    let sig = signal3.msg(&pname, &iname).append1(&*sender);

                    // Two messages will be returned - one is the method return (and should always be there),
                    // and in our case we also have a signal we want to send at the same time.
                    Ok(vec!(mret, sig))
                }).map_err(|e| MethodErr::failed(&e))

            // Our method has one output argument, no input arguments.
            }).outarg::<&str,_>("reply")

        // We also add the signal to the interface. This is mainly for introspection.
        ).add_s(signal2)
    ));

    // We register all object paths in the tree.
    tree.set_registered(&c, true).unwrap();

/*    let mut core = Core::new().unwrap();
    let dbus = TokioServer::new(dbus, core.handle());

    // This dbus_events future contains a poll implementation that pulls messages
    // off the wire and dispatches them to the above-defined callbacks to create futures
    core.run(dbus).unwrap(); */
}

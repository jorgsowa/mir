===description===
Unbraced `namespace X;` sections resolve against their own namespace.
===file===
<?php
namespace A;
class Real {}

namespace B;
class Own {}
function g(): Own {
    new Real();
//      ^^^^ UndefinedClass: Class B\Real does not exist
    return new Own();
}
===expect===

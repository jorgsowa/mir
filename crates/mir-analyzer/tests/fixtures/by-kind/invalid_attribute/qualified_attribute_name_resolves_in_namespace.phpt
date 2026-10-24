===description===
`#[Foo\Attr]` in `namespace App` names `App\Foo\Attr`, even when a global
`Foo\Attr` attribute class exists.
===file===
<?php
namespace Foo {
    #[\Attribute] class Attr {}
}
namespace App\Foo {
    class Attr {}
}
namespace App {
    #[Foo\Attr]
//    ^^^^^^^^ InvalidAttribute: Class Attr does not have an #[Attribute] annotation
    final class K {}
}

===description===
A class whose traits declare the same property incompatibly is a compile error; it never gets constructed, so no MissingConstructor is added. Compatible trait properties still need a constructor.
===file===
<?php
trait LeftTrait { public string $prop; }
trait RightTrait { public int $prop; }
trait SameTrait { public string $prop; }

final class Composed {
    use LeftTrait;
    use RightTrait;
//      ^^^^^^^^^^^ CompileError: LeftTrait and RightTrait define the same property ($prop) in the composition of Composed. However, the definition differs and is considered incompatible. Class was composed [trait-property-conflict]
}

final class Compatible {
//    ^^^^^^^^^^^^^^^^^^ MissingConstructor: Class Compatible has uninitialized properties but no constructor
    use LeftTrait;
    use SameTrait;
}

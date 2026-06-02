===description===
Cannot override final type
===file===
<?php
class P {
    public final function f() : void {}
}

class C extends P {
    public function f() : void {}
}
===expect===
FinalMethodOverridden@7:4-7:33: Method C::f() cannot override final method from P

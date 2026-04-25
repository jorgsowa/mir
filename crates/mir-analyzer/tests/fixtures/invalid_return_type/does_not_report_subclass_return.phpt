===file===
<?php
class Base {}
class Child extends Base {}
function f(): Base {
    return new Child();
}
===expect===

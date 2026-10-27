===description===
Before PHP 8.0 promotion itself is the error; repeated modifiers on the parameter add nothing.
===config===
<mir>
  <phpVersion>7.4</phpVersion>
</mir>
===file===
<?php
class A {
    public function __construct(public private $a) {}
//                              ^^^^^^ ParseError: 'constructor property promotion' requires PHP 8.0 or higher [version-too-low]
//                              ^^^^^^^^^^^^^^^^^ MissingPropertyType: Property A::$a has no type annotation
}

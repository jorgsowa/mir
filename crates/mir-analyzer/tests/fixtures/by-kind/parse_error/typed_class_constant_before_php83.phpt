===description===
Typed class constants require PHP 8.3.
===config===
<mir>
  <phpVersion>8.2</phpVersion>
</mir>
===file===
<?php
class A {
    const string NAME = "a";
//        ^^^^^^ ParseError: 'typed class constants' requires PHP 8.3 or higher [version-too-low]
}

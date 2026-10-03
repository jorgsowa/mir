===description===
__toString() with string|int union return type fires — not all atoms are string
===config===
<mir>
  <phpVersion>8.0</phpVersion>
</mir>
===file===
<?php
class UnionReturn {
    /** @return string|int */
    public function __toString() {
//                               ^ +2:5 InvalidToString: Method UnionReturn::__toString() must return a string
        return 42;
    }
}
new UnionReturn();
===expect===

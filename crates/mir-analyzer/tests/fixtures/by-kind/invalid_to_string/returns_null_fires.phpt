===description===
__toString() with a null return type fires InvalidToString (null is not a string)
===config===
<mir>
  <phpVersion>8.0</phpVersion>
</mir>
===file===
<?php
class NullReturn {
    public function __toString(): null {
//                                     ^ +2:5 InvalidToString: Method NullReturn::__toString() must return a string
        return null;
    }
}
new NullReturn();
===expect===

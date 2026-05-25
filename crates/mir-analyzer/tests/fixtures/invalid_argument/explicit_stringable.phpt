===description===
Explicit stringable
===file===

                    <?php
class A implements Stringable {
    public function __toString(): string {
        return "";
    }
}

===expect===
MissingOverrideAttribute
===ignore===
TODO

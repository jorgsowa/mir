===description===
Explicit stringable
===ignore===
TODO
===file===

                    <?php
class A implements Stringable {
    public function __toString(): string {
        return "";
    }
}

===expect===

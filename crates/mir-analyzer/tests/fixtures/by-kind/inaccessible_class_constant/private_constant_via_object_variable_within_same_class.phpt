===description===
$other::SECRET (object-instance receiver) accessed from within the declaring class must not be reported inaccessible, even though the receiver isn't $this.
===file===
<?php
class Config {
    private const SECRET = "hidden";

    public function reveal(Config $other): string {
        return $other::SECRET;
    }
}
===expect===

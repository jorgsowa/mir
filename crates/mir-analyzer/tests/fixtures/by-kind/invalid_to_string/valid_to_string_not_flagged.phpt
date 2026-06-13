===description===
InvalidToString does NOT fire when __toString returns string.
===file===
<?php
class Label {
    public function __toString(): string {
        return 'label';
    }
}
===expect===

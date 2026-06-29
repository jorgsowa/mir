===description===
__toString() with ?string (string|null) return type fires — null atom is not a string and PHP requires a non-nullable string
===file===
<?php
class NullableReturn {
    public function __toString(): ?string {
        return null;
    }
}
new NullableReturn();
===expect===
InvalidToString@3:42-5:43: Method NullableReturn::__toString() must return a string

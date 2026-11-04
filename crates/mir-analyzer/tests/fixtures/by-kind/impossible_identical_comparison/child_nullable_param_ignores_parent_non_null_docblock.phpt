===description===
A child that widens a parameter to `?T` keeps `null`, even when the parent's `@param` says `T`
===file===
<?php
interface Translator {
    /** @param string $id */
    public function trans(string $id): string;
}

final class Impl implements Translator {
    public function trans(?string $id): string {
        if (null === $id) {
            return '';
        }
        return $id;
    }
}

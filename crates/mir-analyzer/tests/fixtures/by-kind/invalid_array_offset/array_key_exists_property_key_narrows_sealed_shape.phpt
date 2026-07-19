===description===
array_key_exists($this->key, $arr) resolves the key when it's a property
already narrowed to a single literal, same as a plain variable already
does — literal_key resolution only tried extract_var_name.
===config===
suppress=UnusedVariable,UnusedParam,MissingConstructor
===file===
<?php
class Holder {
    /** @var 'favicon' */
    public string $key = 'favicon';

    /** @var string */
    public string $unnarrowedKey = 'x';

    /** @param array{title: string} $meta */
    public function guardedByPropertyStringKey(array $meta): string {
        return array_key_exists($this->key, $meta) ? (string) $meta['favicon'] : '';
    }

    /** @param array{title: string} $meta */
    public function notNarrowedWhenKeyIsNotALiteral(array $meta): string {
        return array_key_exists($this->unnarrowedKey, $meta) ? (string) $meta['favicon'] : '';
    }
}
===expect===
NonExistentArrayOffset@16:78-16:87: Array offset 'favicon' does not exist

===description===
array_key_exists($this->key, $arr) resolves the key when it's a property
already narrowed to a single literal, same as a plain variable already
does — literal_key resolution only tried extract_var_name.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <MissingConstructor errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//                                                                            ^^^^^^^^^ NonExistentArrayOffset: Array offset 'favicon' does not exist
    }
}

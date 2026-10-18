===description===
A stub `@param` that refines the `#[LanguageLevelTypeAware]` type keeps its
template types (ArrayIterator::offsetSet's `TValue`). One that leaves out an
atom the attribute adds (`iterator_count`'s `Traversable` before 8.2 added
`array`) or merely restates it stays the attribute type, so an override that
widens the param (`string|DateTimeZone|null`) isn't narrowed back to the stub's.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function setValues(): void {
    $it = new ArrayIterator(['a' => 1]);
    $it['b'] = 2;
    $it->offsetSet('c', 'x');
}

function countsArrays(): int {
    return iterator_count([1, 2]);
}

final class LocalDateTime extends DateTime {
    public static function createFromFormat(
        string $format,
        string $datetime,
        string|DateTimeZone|null $timezone = null,
    ): static|false {
        if (is_string($timezone)) {
            $timezone = new DateTimeZone($timezone);
        }
        return false;
    }
}
===expect===
InvalidArgument@5:24-5:27: Argument $value of offsetSet() expects 'int', got '"x"'

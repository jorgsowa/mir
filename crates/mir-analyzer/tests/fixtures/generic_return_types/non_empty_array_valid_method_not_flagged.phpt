===file===
<?php
/**
 * @template T
 */
class Wrapper {
    /** @return non-empty-array<string, T> */
    public function asMap(): array { return []; }
}
class Tag { public function label(): string { return 'x'; } }
function test(): void {
    /** @var Wrapper<Tag> $w */
    $w = new Wrapper();
    foreach ($w->asMap() as $tag) {
        $tag->label();
    }
}
===expect===

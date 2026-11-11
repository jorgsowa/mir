===description===
Side effects inside a nested closure or a nested declaration do not make the outer closure impure.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param pure-callable $factory */
function build($factory): void {}

build(static function (): \Closure {
    return static function (): void {
        echo 'later';
    };
});
build(static fn (): \Closure => static fn (): int => print 'later');
build(static function (): object {
    return new class {
        public function run(): void {
            echo 'method';
        }
    };
});

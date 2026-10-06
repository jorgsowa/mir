===description===
A callable-array pair (keyed array [$obj, 'method']) satisfies a callable parameter type
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Handler {
    public function handle(): void {}
}

function dispatch(callable $fn): void {
    $fn();
}

$h = new Handler();
dispatch([$h, 'handle']);

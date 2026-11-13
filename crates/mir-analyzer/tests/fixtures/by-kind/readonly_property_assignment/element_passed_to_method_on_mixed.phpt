===description===
A readonly property's element passed to a method on `mixed` isn't assumed to be written.
===config===
<mir>
  <issueHandlers>
    <MixedMethodCall errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class R {
    public function __construct(public readonly array $context = []) {}
}
function h(R $r, mixed $m): void {
    $m->dispatch($r->context['e']);
}

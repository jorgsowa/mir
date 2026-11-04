===description===
On `A&I`, a method `A` declares is checked against `A`'s signature, not an `@method` on `I`
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Client {
    public function evalSha(string $sha, array $args = []): mixed { return null; }
}

/** @method mixed evalsha(string $script, int $numkeys) */
interface Predis {}

function f(Client|Predis $r): void {
    if ($r instanceof Client) {
        $r->evalSha('x', ['k']);
    }
}

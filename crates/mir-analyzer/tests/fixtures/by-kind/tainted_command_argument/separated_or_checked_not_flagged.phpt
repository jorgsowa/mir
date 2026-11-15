===description===
Arguments after `--`, arguments that start with a literal, and values checked not
to start with `-` can't pass an option.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function separated(): void {
    $spec = [];
    proc_open(['git', 'clone', '--', $_GET['repo']], $spec, $pipes);
}

function prefixed(): void {
    $spec = [];
    proc_open(['git', 'checkout', '--branch=' . $_GET['b']], $spec, $pipes);
}

function rejected(): void {
    $spec = [];
    $repo = (string) $_GET['repo'];
    if (str_starts_with($repo, '-')) {
        throw new InvalidArgumentException('bad repo');
    }
    proc_open(['git', 'clone', $repo], $spec, $pipes);
}

function firstCharacter(): void {
    $spec = [];
    $host = (string) $_GET['host'];
    if ($host[0] === '-') {
        return;
    }
    proc_open(['ssh', $host], $spec, $pipes);
}

function schemeRequired(): void {
    $spec = [];
    $url = (string) $_GET['url'];
    if (!str_starts_with($url, 'https://')) {
        return;
    }
    proc_open(['git', 'clone', $url], $spec, $pipes);
}

function viaStrpos(): void {
    $spec = [];
    $name = (string) $_GET['name'];
    if (strpos($name, '-') !== 0) {
        proc_open(['touch', $name], $spec, $pipes);
    }
}

function programControlled(string $branch): void {
    $spec = [];
    proc_open(['git', 'checkout', $branch], $spec, $pipes);
}

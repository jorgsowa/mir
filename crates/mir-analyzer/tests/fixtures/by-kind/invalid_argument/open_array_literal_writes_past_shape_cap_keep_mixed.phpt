===description===
Literal-key writes onto an untyped array stay open past the shape key cap, so unwritten keys remain mixed
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Repo
{
    public function row(): array { return []; }
    public function take(int $id): void { echo $id; }
}

function writes_within_cap(Repo $repo): void {
    $row = $repo->row();
    $row['k0'] = 's';
    $row['k1'] = true;
    /** @mir-check $row['id'] is mixed */
    $repo->take($row['id']);
}

function writes_past_cap(Repo $repo): void {
    $row = $repo->row();
    $row['k0'] = 's';
    $row['k1'] = true;
    $row['k2'] = 's';
    $row['k3'] = true;
    $row['k4'] = 's';
    $row['k5'] = true;
    $row['k6'] = 's';
    $row['k7'] = true;
    $row['k8'] = 's';
    $row['k9'] = true;
    /** @mir-check $row['id'] is mixed */
    $repo->take($row['id']);
}

function overwrite_after_cap(Repo $repo): void {
    $row = $repo->row();
    $row['k0'] = 's';
    $row['k1'] = true;
    $row['k2'] = 's';
    $row['k3'] = true;
    $row['k4'] = 's';
    $row['k5'] = true;
    $row['k6'] = 's';
    $row['k7'] = true;
    $row['k8'] = 's';
    $row['k9'] = true;
    $row['k0'] = 1;
    /** @mir-check $row['id'] is mixed */
    $repo->take($row['id']);
}

function dynamic_key_after_cap(Repo $repo, string $name): void {
    $row = $repo->row();
    $row['k0'] = 's';
    $row['k1'] = true;
    $row['k2'] = 's';
    $row['k3'] = true;
    $row['k4'] = 's';
    $row['k5'] = true;
    $row['k6'] = 's';
    $row['k7'] = true;
    $row['k8'] = 's';
    $row[$name] = true;
    $row['k9'] = true;
    /** @mir-check $row['id'] is mixed */
    $repo->take($row['id']);
}

function push_after_cap(Repo $repo): void {
    $row = $repo->row();
    $row['k0'] = 's';
    $row['k1'] = true;
    $row['k2'] = 's';
    $row['k3'] = true;
    $row['k4'] = 's';
    $row['k5'] = true;
    $row['k6'] = 's';
    $row['k7'] = true;
    $row['k8'] = 's';
    $row['k9'] = true;
    $row[] = 'pushed';
    /** @mir-check $row[0] is mixed */
    $repo->take($row[0]);
}
===expect===

===description===
Tainted llm prompt from user input
===config===
suppress=MixedArrayAccess,UnusedParam
===file===
<?php
class LlmAgent {
    /** @taint-sink llm_prompt $prompt */
    public function prompt(string $prompt): string {
        return "";
    }
}

$agent = new LlmAgent();
$agent->prompt((string) $_GET["question"]);
===expect===
TaintedLlmPrompt@10:0-10:42: Tainted LLM prompt — possible prompt injection

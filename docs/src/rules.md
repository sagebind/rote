# File rules

A _rule_ is a powerful way to dynamically define tasks based on a set of input and output files. Rules are not tasks, but act as a template for tasks to be automatically generated.

## Defining rules

To define a new rule, we use the `rule()` function:

```lua
rule("foo.txt", function(output)
    fs.put(output, "foo")
end)
```

This looks similar to a task definition, but has a couple of important differences. The first difference is in the name of the rule. When creating a task, the name acts as a canonical identifier for that task and is used to recall that task from the command line. In rules, the name of the rule is also the name of the *output* file that the rule produces.

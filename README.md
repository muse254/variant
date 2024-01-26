<div align="center">
	<img width="256" src="assets/variant.svg" alt="Variant logo">

This tool addresses the challenge of managing multiple Git profiles, such as work and personal, by offering a convenient and efficient means of switching between them.

</div>

## Installation

It is important to note that `variant` assumes that you have already have [git](https://git-scm.com/) installed and configured. Also the `ssh-add` command is expected to be available in your PATH, [this is what it does](https://superuser.com/questions/360686/what-exactly-does-ssh-add-do).

## Usage

It assumes a directory structure that looks like this for the git accounts to be managed, `foo` and `bar`:

```bash
|-- ~/.ssh
|	|-- foo
|	|	|-- config
|	|	|-- id_rsa
|	|	|-- id_rsa.pub
|	|-- bar
|	|	|-- config
|	|	|-- id_rsa
|	|	|-- id_rsa.pub
```

It's important to note that the name of the folder and the name of the git profile match, in this case `foo` and `bar`. When switching profiles, variant will try to pig the
server 

When I create a new repository, I can specify which account to use:

```bash
cd my-awesome-project # We navigate to out project directory
variant var -n foo # We specify which account to use, assuming variant is in PATH
```

This changes the global git configuration to use the `foo` account. If you need to only configure an account for the current repository,
you can use the `--sacred` flag:

```bash
cd my-awesome-project
variant var -n foo --sacred
```

We can also query for information about the profile configured:

```bash
variant whoami -v
```

## Other

The project is really a wrapper around `git` and the `ssh` commands, so it's not really doing anything special. The selling point is ergonomics.

With aliasing magic, I can do this:

```bash
alias vf="variant var -n foo"
alias vb="variant var -n bar"
vf # Switch to foo
vb # Switch to bar
```

With some imagination, switching accounts becomes a breeze.

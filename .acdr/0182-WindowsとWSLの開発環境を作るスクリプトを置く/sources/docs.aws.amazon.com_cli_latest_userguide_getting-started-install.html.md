

# Installing or updating to the latest version of the AWS CLI
<a name="getting-started-install"></a>

This topic describes how to install or update the latest release of the AWS Command Line Interface (AWS CLI) on supported operating systems. For information on the latest releases of AWS CLI, see the [AWS CLI version 2 Changelog](https://raw.githubusercontent.com/aws/aws-cli/v2/CHANGELOG.rst) on GitHub.

To install a past release of the AWS CLI, see [Installing past releases of the AWS CLI version 2](getting-started-version.md). For uninstall instructions, see [Uninstalling the AWS CLI version 2](uninstall.md).

**Important**  
AWS CLI versions 1 and 2 use the same `aws` command name. If you previously installed AWS CLI version 1, see [Migration guide for the AWS CLI version 2](cliv2-migration.md).

**Topics**
+ [AWS CLI install instructions](#getting-started-install-instructions)
+ [AWS CLI update instructions](#cli-update)
+ [Troubleshooting AWS CLI install and uninstall errors](#install-tshoot)
+ [Next steps](#install-next-steps)

## AWS CLI install instructions
<a name="getting-started-install-instructions"></a>

For installation instructions, expand the section for your operating system.

### Linux
<a name="install-linux"></a>

#### Install requirements
<a name="install-linux-prereqs"></a>
+ You must be able to extract or "unzip" the downloaded package. If your operating system doesn't have the built-in `unzip` command, use an equivalent.
+ The AWS CLI uses `glibc`, `groff`, and `less`. These are included by default in most major distributions of Linux.
+ We support the AWS CLI on 64-bit versions of recent distributions of CentOS, Fedora, Ubuntu, Amazon Linux 1, Amazon Linux 2, Amazon Linux 2023, and Linux ARM.
+ Because AWS doesn't maintain third-party repositories other than `snap`, we can’t guarantee that they contain the latest version of the AWS CLI.

#### Install the AWS CLI
<a name="install-linux-instructions"></a>

**Warning**  
If this is your first time updating on Amazon Linux, to install the latest version of the AWS CLI, you must uninstall the pre-installed `yum` version using the following command:  

```
$ sudo yum remove awscli
```
After the `yum` installation of the AWS CLI is removed, follow the below Linux install instructions.

You can install the AWS CLI by using one of the following methods:
+ **The install script** is the recommended way to install the AWS CLI. To update the AWS CLI, see [AWS CLI update instructions](#cli-update).
+ **The command line installer** is a good option for version control because you can specify the version to install. This option does not auto-update; to update, see [AWS CLI update instructions](#cli-update).
+ **The officially supported `snap` package** is a good option to always have the latest version of the AWS CLI as snap packages automatically refresh. There is no built-in support for selecting minor versions of AWS CLI and therefore is not an optimal install method if your team needs to pin versions.

------
#### [ Install script (recommended) ]

The install script downloads, verifies, and installs the AWS CLI for Linux in one step. It works for both Linux x86 (64-bit) and Linux ARM, and installs for the current user by default.

**To install** the AWS CLI, run the following command.

```
$ curl -fsSL https://awscli.amazonaws.com/v2/install.sh | bash
```

**Note**  
By default, the script installs to `$HOME/.local/share/aws-cli` and creates a symlink in `$HOME/.local/bin`. To install to different locations, set the `$XDG_DATA_HOME` and `$XDG_BIN_HOME` environment variables before running the script.

To install for all users instead (installs to `/usr/local/aws-cli` and `/usr/local/bin`), which requires `sudo`, pass the `--system` argument to the script.

```
$ curl -fsSL https://awscli.amazonaws.com/v2/install.sh | sudo bash -s -- --system
```

For the full list of options, run the script with `--help`.

```
$ curl -fsSL https://awscli.amazonaws.com/v2/install.sh | bash -s -- --help
```

To update the AWS CLI, see [AWS CLI update instructions](#cli-update).

------
#### [ Command line installer - Linux x86 (64-bit) ]

Follow these steps from the command line to install the AWS CLI on Linux. To update an installation created this way, see [AWS CLI update instructions](#cli-update) instead of repeating these steps.

The following are quick installation steps in a single copy and paste group that provide a basic installation. For guided instructions, see the steps that follow.

**Note**  
**(Optional)** The following command block downloads and installs the AWS CLI without first verifying the integrity of your download. To verify the integrity of your download, use the below step by step instructions.

**To install** the AWS CLI, run the following commands.

```
$ curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"
unzip awscliv2.zip
sudo ./aws/install
```

**Guided installation steps**

1. Download the installation file in one of the following ways:
   + **Use the `curl` command** – The `-o` option specifies the file name that the downloaded package is written to. The options on the following example command write the downloaded file to the current directory with the local name `awscliv2.zip`. 

     ```
     $ curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"
     ```
   + **Downloading from the URL** – To download the installer with your browser, use the following URL: [https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip](https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip) 

1. **(Optional) Verifying the integrity of your downloaded zip file**

   If you chose to manually download the AWS CLI installer package `.zip` in the above steps, you can use the following steps to verify the signatures by using the `GnuPG` tool.

   The AWS CLI installer package `.zip` files are cryptographically signed using PGP signatures. If there is any damage or alteration of the files, this verification fails and you should not proceed with installation.

   1. Download and install the `gpg` command using your package manager. For more information about `GnuPG`, see the [GnuPG website](https://www.gnupg.org/). 

   1. To create the public key file, create a text file and paste in the following text.

      ```
      -----BEGIN PGP PUBLIC KEY BLOCK-----
      
      mQINBF2Cr7UBEADJZHcgusOJl7ENSyumXh85z0TRV0xJorM2B/JL0kHOyigQluUG
      ZMLhENaG0bYatdrKP+3H91lvK050pXwnO/R7fB/FSTouki4ciIx5OuLlnJZIxSzx
      PqGl0mkxImLNbGWoi6Lto0LYxqHN2iQtzlwTVmq9733zd3XfcXrZ3+LblHAgEt5G
      TfNxEKJ8soPLyWmwDH6HWCnjZ/aIQRBTIQ05uVeEoYxSh6wOai7ss/KveoSNBbYz
      gbdzoqI2Y8cgH2nbfgp3DSasaLZEdCSsIsK1u05CinE7k2qZ7KgKAUIcT/cR/grk
      C6VwsnDU0OUCideXcQ8WeHutqvgZH1JgKDbznoIzeQHJD238GEu+eKhRHcz8/jeG
      94zkcgJOz3KbZGYMiTh277Fvj9zzvZsbMBCedV1BTg3TqgvdX4bdkhf5cH+7NtWO
      lrFj6UwAsGukBTAOxC0l/dnSmZhJ7Z1KmEWilro/gOrjtOxqRQutlIqG22TaqoPG
      fYVN+en3Zwbt97kcgZDwqbuykNt64oZWc4XKCa3mprEGC3IbJTBFqglXmZ7l9ywG
      EEUJYOlb2XrSuPWml39beWdKM8kzr1OjnlOm6+lpTRCBfo0wa9F8YZRhHPAkwKkX
      XDeOGpWRj4ohOx0d2GWkyV5xyN14p2tQOCdOODmz80yUTgRpPVQUtOEhXQARAQAB
      tCFBV1MgQ0xJIFRlYW0gPGF3cy1jbGlAYW1hem9uLmNvbT6JAlQEEwEIAD4CGwMF
      CwkIBwIGFQoJCAsCBBYCAwECHgECF4AWIQT7Xbd/1cEYuAURraimMQrMRnJHXAUC
      akV0ygUJDqP4lQAKCRCmMQrMRnJHXFHjD/9eyZLYcKuQOlLvtqSDtUBiEZf6ZZjM
      i3ygYH8rJNtuToUH+HvSpe819urJCquXhDrlK6N+aqW0hCLtNABJG/vsafIgvIYJ
      hSGgpgtNnQyMV1jViRWqPjbouw8OkYKBThUfT1i2Y+wn58ifs6ODBCmTexWtXspA
      Si+Gt49xDOW0APmbOPnI+a4HJW6tVEo6MWS0WjzpiBayR3d1A4pt4YrPfSdDgpLo
      h2SLQqlRqvvVZJaWBjhkErNFpfsBA06sDcPEOb0G8LBUbR4WOcdvhe5LubJbZuxC
      AG9kNPCVeQP1ixwjgjXKysaxeQ6rv0VzIQgRp6tLVLWhy6AKDNvLjFSsmXZ1Wl08
      Y/RlOHXlzLuQMRE6sR1wOdRxc9TsrNWTGiBK65cvSWOy03JeBkQQ8pesqltiyxI9
      U21kkgiXtTSKNGfKK8pO27D81YANhRqPK7iTp6kuFiY2WtOg90KTMNlIT+Ff85Y2
      b1rHj6Z0SrCkJujhWk3IBPic/wJgz01LEc/OAdUPlby90RJZcIBhSlWhT7mXnXIO
      c0HWlNQrns2s3CTyYwZSiSlYe9ApeLwhjDo8NhbFuCAy61l6O5UsR4AfZxx/rGKv
      2wFb1/RN/P4gNe6vmxZAPjR0AQcwD3tc2McimOLr/22kmPz8IH3I0X7WoSFr0Biz
      E91G7bb0hOb/cA==
      =knv7
      -----END PGP PUBLIC KEY BLOCK-----
      ```

      For reference, the following are the details of the public key.

      ```
      Key ID:           A6310ACC4672475C
      Type:             RSA
      Size:             4096/4096
      Created:          2019-09-18
      Expires:          2027-07-01
      User ID:          AWS CLI Team <aws-cli@amazon.com>
      Key fingerprint:  FB5D B77F D5C1 18B8 0511  ADA8 A631 0ACC 4672 475C
      ```

   1. Import the AWS CLI public key with the following command, substituting {{public-key-file-name}} with the file name of the public key you created.

      ```
      $ gpg --import {{public-key-file-name}}
      gpg: /home/{{username}}/.gnupg/trustdb.gpg: trustdb created
      gpg: key A6310ACC4672475C: public key "AWS CLI Team <aws-cli@amazon.com>" imported
      gpg: Total number processed: 1
      gpg:               imported: 1
      ```

   1. Download the AWS CLI signature file for the package you downloaded. It has the same path and name as the `.zip` file it corresponds to, but has the extension `.sig`. In the following examples, we save it to the current directory as a file named `awscliv2.sig`.

      **For the latest version of the AWS CLI,** use the following command block:

      ```
      $ curl -o awscliv2.sig https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip.sig
      ```

   1. Verify the signature, passing both the downloaded `.sig` and `.zip` file names as parameters to the `gpg` command.

      ```
      $ gpg --verify awscliv2.sig awscliv2.zip
      ```

      The output should look similar to the following.

      ```
      gpg: Signature made Mon Nov  4 19:00:01 2019 PST
      gpg:                using RSA key FB5D B77F D5C1 18B8 0511 ADA8 A631 0ACC 4672 475C
      gpg: Good signature from "AWS CLI Team <aws-cli@amazon.com>" [unknown]
      gpg: WARNING: This key is not certified with a trusted signature!
      gpg:          There is no indication that the signature belongs to the owner.
      Primary key fingerprint: FB5D B77F D5C1 18B8 0511  ADA8 A631 0ACC 4672 475C
      ```
**Important**  
The warning in the output is expected and doesn't indicate a problem. It occurs because there isn't a chain of trust between your personal PGP key (if you have one) and the AWS CLI PGP key. For more information, see [Web of trust](https://en.wikipedia.org/wiki/Web_of_trust).

1. Unzip the installer. If your Linux distribution doesn't have a built-in `unzip` command, use an equivalent to unzip it. The following example command unzips the package and creates a directory named `aws` under the current directory.

   ```
   $ unzip awscliv2.zip
   ```
**Note**  
When updating from a previous version, the `unzip` command prompts to overwrite existing files. To skip these prompts, such as with script automation, use the `-u` update flag for `unzip`. This flag automatically updates existing files and creates new ones as needed.  

   ```
   $ unzip -u awscliv2.zip
   ```

1. Run the install program. The installation command uses a file named `install` in the newly unzipped `aws` directory. By default, the files are all installed to `/usr/local/aws-cli`, and a symbolic link is created in `/usr/local/bin`. The command includes `sudo` to grant write permissions to those directories. 

   ```
   $ sudo ./aws/install
   ```

   You can install without `sudo` if you specify directories that you already have write permissions to. Use the following instructions for the `install` command to specify the installation location:
   + Ensure that the paths you provide to the `-i` and `-b` parameters contain no volume name or directory names that contain any space characters or other white space characters. If there is a space, the installation fails.
   + `--install-dir` or `-i` – This option specifies the directory to copy all of the files to.

     The default value is `/usr/local/aws-cli`.
   + `--bin-dir` or `-b` – This option specifies that the main `aws` program in the install directory is symbolically linked to the file `aws` in the specified path. You must have write permissions to the specified directory. Creating a symlink to a directory that is already in your path eliminates the need to add the install directory to the user's `$PATH` variable. 

     The default value is `/usr/local/bin`.

   ```
   $ ./aws/install -i {{/usr/local/aws-cli}} -b {{/usr/local/bin}}
   ```
**Note**  
To update an installation created this way, see [AWS CLI update instructions](#cli-update).

1. Confirm the installation with the following command. 

   ```
   $ aws --version
   aws-cli/2.27.41 Python/3.11.6 Linux/5.10.205-195.807.amzn2.x86_64
   ```

   If the `aws` command cannot be found, you might need to restart your terminal or follow the troubleshooting in [Troubleshooting errors for the AWS CLI](cli-chap-troubleshooting.md).

------
#### [ Command line - Linux ARM ]

Follow these steps from the command line to install the AWS CLI on Linux. To update an installation created this way, see [AWS CLI update instructions](#cli-update) instead of repeating these steps.

The following are quick installation steps in a single copy and paste group that provide a basic installation. For guided instructions, see the steps that follow.

**Note**  
**(Optional)** The following command block downloads and installs the AWS CLI without first verifying the integrity of your download. To verify the integrity of your download, use the below step by step instructions.

**To install** the AWS CLI, run the following commands.

```
$ curl "https://awscli.amazonaws.com/awscli-exe-linux-aarch64.zip" -o "awscliv2.zip"
unzip awscliv2.zip
sudo ./aws/install
```

**Guided installation steps**

1. Download the installation file in one of the following ways:
   + **Use the `curl` command** – The `-o` option specifies the file name that the downloaded package is written to. The options on the following example command write the downloaded file to the current directory with the local name `awscliv2.zip`. 

     ```
     $ curl "https://awscli.amazonaws.com/awscli-exe-linux-aarch64.zip" -o "awscliv2.zip"
     ```
   + **Downloading from the URL** – To download the installer with your browser, use the following URL: [https://awscli.amazonaws.com/awscli-exe-linux-aarch64.zip](https://awscli.amazonaws.com/awscli-exe-linux-aarch64.zip) 

1. **(Optional) Verifying the integrity of your downloaded zip file**

   If you chose to manually download the AWS CLI installer package `.zip` in the above steps, you can use the following steps to verify the signatures by using the `GnuPG` tool.

   The AWS CLI installer package `.zip` files are cryptographically signed using PGP signatures. If there is any damage or alteration of the files, this verification fails and you should not proceed with installation.

   1. Download and install the `gpg` command using your package manager. For more information about `GnuPG`, see the [GnuPG website](https://www.gnupg.org/). 

   1. To create the public key file, create a text file and paste in the following text.

      ```
      -----BEGIN PGP PUBLIC KEY BLOCK-----
      
      mQINBF2Cr7UBEADJZHcgusOJl7ENSyumXh85z0TRV0xJorM2B/JL0kHOyigQluUG
      ZMLhENaG0bYatdrKP+3H91lvK050pXwnO/R7fB/FSTouki4ciIx5OuLlnJZIxSzx
      PqGl0mkxImLNbGWoi6Lto0LYxqHN2iQtzlwTVmq9733zd3XfcXrZ3+LblHAgEt5G
      TfNxEKJ8soPLyWmwDH6HWCnjZ/aIQRBTIQ05uVeEoYxSh6wOai7ss/KveoSNBbYz
      gbdzoqI2Y8cgH2nbfgp3DSasaLZEdCSsIsK1u05CinE7k2qZ7KgKAUIcT/cR/grk
      C6VwsnDU0OUCideXcQ8WeHutqvgZH1JgKDbznoIzeQHJD238GEu+eKhRHcz8/jeG
      94zkcgJOz3KbZGYMiTh277Fvj9zzvZsbMBCedV1BTg3TqgvdX4bdkhf5cH+7NtWO
      lrFj6UwAsGukBTAOxC0l/dnSmZhJ7Z1KmEWilro/gOrjtOxqRQutlIqG22TaqoPG
      fYVN+en3Zwbt97kcgZDwqbuykNt64oZWc4XKCa3mprEGC3IbJTBFqglXmZ7l9ywG
      EEUJYOlb2XrSuPWml39beWdKM8kzr1OjnlOm6+lpTRCBfo0wa9F8YZRhHPAkwKkX
      XDeOGpWRj4ohOx0d2GWkyV5xyN14p2tQOCdOODmz80yUTgRpPVQUtOEhXQARAQAB
      tCFBV1MgQ0xJIFRlYW0gPGF3cy1jbGlAYW1hem9uLmNvbT6JAlQEEwEIAD4CGwMF
      CwkIBwIGFQoJCAsCBBYCAwECHgECF4AWIQT7Xbd/1cEYuAURraimMQrMRnJHXAUC
      akV0ygUJDqP4lQAKCRCmMQrMRnJHXFHjD/9eyZLYcKuQOlLvtqSDtUBiEZf6ZZjM
      i3ygYH8rJNtuToUH+HvSpe819urJCquXhDrlK6N+aqW0hCLtNABJG/vsafIgvIYJ
      hSGgpgtNnQyMV1jViRWqPjbouw8OkYKBThUfT1i2Y+wn58ifs6ODBCmTexWtXspA
      Si+Gt49xDOW0APmbOPnI+a4HJW6tVEo6MWS0WjzpiBayR3d1A4pt4YrPfSdDgpLo
      h2SLQqlRqvvVZJaWBjhkErNFpfsBA06sDcPEOb0G8LBUbR4WOcdvhe5LubJbZuxC
      AG9kNPCVeQP1ixwjgjXKysaxeQ6rv0VzIQgRp6tLVLWhy6AKDNvLjFSsmXZ1Wl08
      Y/RlOHXlzLuQMRE6sR1wOdRxc9TsrNWTGiBK65cvSWOy03JeBkQQ8pesqltiyxI9
      U21kkgiXtTSKNGfKK8pO27D81YANhRqPK7iTp6kuFiY2WtOg90KTMNlIT+Ff85Y2
      b1rHj6Z0SrCkJujhWk3IBPic/wJgz01LEc/OAdUPlby90RJZcIBhSlWhT7mXnXIO
      c0HWlNQrns2s3CTyYwZSiSlYe9ApeLwhjDo8NhbFuCAy61l6O5UsR4AfZxx/rGKv
      2wFb1/RN/P4gNe6vmxZAPjR0AQcwD3tc2McimOLr/22kmPz8IH3I0X7WoSFr0Biz
      E91G7bb0hOb/cA==
      =knv7
      -----END PGP PUBLIC KEY BLOCK-----
      ```

      For reference, the following are the details of the public key.

      ```
      Key ID:           A6310ACC4672475C
      Type:             RSA
      Size:             4096/4096
      Created:          2019-09-18
      Expires:          2027-07-01
      User ID:          AWS CLI Team <aws-cli@amazon.com>
      Key fingerprint:  FB5D B77F D5C1 18B8 0511  ADA8 A631 0ACC 4672 475C
      ```

   1. Import the AWS CLI public key with the following command, substituting {{public-key-file-name}} with the file name of the public key you created.

      ```
      $ gpg --import {{public-key-file-name}}
      gpg: /home/{{username}}/.gnupg/trustdb.gpg: trustdb created
      gpg: key A6310ACC4672475C: public key "AWS CLI Team <aws-cli@amazon.com>" imported
      gpg: Total number processed: 1
      gpg:               imported: 1
      ```

   1. Download the AWS CLI signature file for the package you downloaded. It has the same path and name as the `.zip` file it corresponds to, but has the extension `.sig`. In the following examples, we save it to the current directory as a file named `awscliv2.sig`.

      **For the latest version of the AWS CLI, **use the following command block:

      ```
      $ curl -o awscliv2.sig https://awscli.amazonaws.com/awscli-exe-linux-aarch64.zip.sig
      ```

   1. Verify the signature, passing both the downloaded `.sig` and `.zip` file names as parameters to the `gpg` command.

      ```
      $ gpg --verify awscliv2.sig awscliv2.zip
      ```

      The output should look similar to the following.

      ```
      gpg: Signature made Mon Nov  4 19:00:01 2019 PST
      gpg:                using RSA key FB5D B77F D5C1 18B8 0511 ADA8 A631 0ACC 4672 475C
      gpg: Good signature from "AWS CLI Team <aws-cli@amazon.com>" [unknown]
      gpg: WARNING: This key is not certified with a trusted signature!
      gpg:          There is no indication that the signature belongs to the owner.
      Primary key fingerprint: FB5D B77F D5C1 18B8 0511  ADA8 A631 0ACC 4672 475C
      ```
**Important**  
The warning in the output is expected and doesn't indicate a problem. It occurs because there isn't a chain of trust between your personal PGP key (if you have one) and the AWS CLI PGP key. For more information, see [Web of trust](https://en.wikipedia.org/wiki/Web_of_trust).

1. Unzip the installer. If your Linux distribution doesn't have a built-in `unzip` command, use an equivalent to unzip it. The following example command unzips the package and creates a directory named `aws` under the current directory.

   ```
   $ unzip awscliv2.zip
   ```
**Note**  
When updating from a previous version, the `unzip` command prompts to overwrite existing files. To skip these prompts, such as with script automation, use the `-u` update flag for `unzip`. This flag automatically updates existing files and creates new ones as needed.  

   ```
   $ unzip -u awscliv2.zip
   ```

1. Run the install program. The installation command uses a file named `install` in the newly unzipped `aws` directory. By default, the files are all installed to `/usr/local/aws-cli`, and a symbolic link is created in `/usr/local/bin`. The command includes `sudo` to grant write permissions to those directories. 

   ```
   $ sudo ./aws/install
   ```

   You can install without `sudo` if you specify directories that you already have write permissions to. Use the following instructions for the `install` command to specify the installation location:
   + Ensure that the paths you provide to the `-i` and `-b` parameters contain no volume name or directory names that contain any space characters or other white space characters. If there is a space, the installation fails.
   + `--install-dir` or `-i` – This option specifies the directory to copy all of the files to.

     The default value is `/usr/local/aws-cli`.
   + `--bin-dir` or `-b` – This option specifies that the main `aws` program in the install directory is symbolically linked to the file `aws` in the specified path. You must have write permissions to the specified directory. Creating a symlink to a directory that is already in your path eliminates the need to add the install directory to the user's `$PATH` variable. 

     The default value is `/usr/local/bin`.

   ```
   $ ./aws/install -i {{/usr/local/aws-cli}} -b {{/usr/local/bin}}
   ```
**Note**  
To update an installation created this way, see [AWS CLI update instructions](#cli-update).

1. Confirm the installation with the following command. 

   ```
   $ aws --version
   aws-cli/2.27.41 Python/3.11.6 Linux/5.10.205-195.807.amzn2.x86_64
   ```

   If the `aws` command cannot be found, you might need to restart your terminal or follow the troubleshooting in [Troubleshooting errors for the AWS CLI](cli-chap-troubleshooting.md).

------
#### [ Snap package ]

We provide an official AWS supported version of the AWS CLI on `snap`. If you want to always have the latest version of the AWS CLI installed on your system, a snap package provides this for you as it auto-updates. There is no built-in support for selecting minor versions of AWS CLI and therefore it is not an optimal install method if your team needs to pin versions. If you want to install a specific minor version of the AWS CLI, we suggest you use the command line installer.

1. If your Linux platform does not already have `snap` installed, install `snap` on your platform. 

   1. For information on installing `snap`, see [Installing the daemon](https://snapcraft.io/docs/installing-snapd) in the *Snap documentation*.

   1. You may need to restart your system so that your `PATH` variables are updated correctly. If you are having installation issues, follow steps in [Fix common issues](https://snapcraft.io/docs/fix-common-issues) in the *Snap documentation*.

   1. To verify that `snap` is installed correctly, run the following command.

      ```
      $ snap version
      ```

1. Run the following `snap install` command for the AWS CLI.

   ```
   $ snap install aws-cli --classic
   ```

   Depending on your permissions, you may need to add `sudo` to the command.

   ```
   $ sudo snap install aws-cli --classic
   ```
**Note**  
To view the snap repository for the AWS CLI, including additional `snap` instructions, see the [`aws-cli`](https://snapcraft.io/aws-cli) page in the *Canonical Snapcraft website*.

1. Verify that the AWS CLI installed correctly.

   ```
   $ aws --version
   aws-cli/2.27.41 Python/3.11.6 Linux/5.10.205-195.807.amzn2.x86_64
   ```

   If you get an error, see [Troubleshooting errors for the AWS CLI](cli-chap-troubleshooting.md).

------

### macOS
<a name="install-macos"></a>

#### Install requirements
<a name="install-macos-prereqs"></a>
+ We support the AWS CLI on macOS versions 11 and later. For more information, see [macOS support policy updates for the AWS CLI v2](https://aws.amazon.com/blogs/developer/macos-support-policy-updates-for-the-aws-cli-v2/) on the *AWS Developer Tools Blog*.
+ Because AWS doesn't maintain third-party repositories, we can’t guarantee that they contain the latest version of the AWS CLI.

**macOS version support matrix**


| AWS CLI version | Supported macOS version | 
| --- | --- | 
| 2.21.0 – current | 11\+ | 
| 2.17.0 –2.20.0 | 10.15\+ | 
| 2.0.0 – 2.16.12 | 10.14 and below | 

#### Install the AWS CLI
<a name="install-macos0-instructions"></a>

You can install the AWS CLI on macOS in the following ways. The install script is the recommended method. To update an existing installation, see [AWS CLI update instructions](#cli-update).

------
#### [ Install script (recommended) ]

The install script downloads, verifies, and installs the AWS CLI for macOS in one step. It installs for the current user by default.

**To install** the AWS CLI, run the following command.

```
$ curl -fsSL https://awscli.amazonaws.com/v2/install.sh | bash
```

**Note**  
By default, the script installs to `$HOME/.local/share/aws-cli` and creates a symlink in `$HOME/.local/bin`. To install to different locations, set the `$XDG_DATA_HOME` and `$XDG_BIN_HOME` environment variables before running the script.

To install for all users instead (installs to `/usr/local/aws-cli` and `/usr/local/bin`), which requires `sudo`, pass the `--system` argument to the script.

```
$ curl -fsSL https://awscli.amazonaws.com/v2/install.sh | sudo bash -s -- --system
```

For the full list of options, run the script with `--help`.

```
$ curl -fsSL https://awscli.amazonaws.com/v2/install.sh | bash -s -- --help
```

To update the AWS CLI, see [AWS CLI update instructions](#cli-update).

------
#### [ GUI installer ]

The following steps show how to install the latest version of the AWS CLI by using the standard macOS user interface and your browser.

1. In your browser, download the macOS `pkg` file: [https://awscli.amazonaws.com/AWSCLIV2.pkg](https://awscli.amazonaws.com/AWSCLIV2.pkg) 

1. Run your downloaded file and follow the on-screen instructions. You can choose to install the AWS CLI in the following ways:
   + **For all users on the computer (requires `sudo`)**
     + You can install to any folder, or choose the recommended default folder of `/usr/local/aws-cli`.
     + The installer automatically creates a symlink at `/usr/local/bin/aws` that links to the main program in the installation folder you chose.
   + **For only the current user (doesn't require `sudo`)**
     + You can install to any folder to which you have write permission.
     + Due to standard user permissions, after the installer finishes, you must manually create a symlink file in your `$PATH` that points to the `aws` and `aws_completer` programs by using the following commands at the command prompt. The default location for a symlink is `/usr/local/bin/`:

       ```
       $ ln -s /{{folder/installed}}/aws-cli/aws /{{usr/local/bin}}/aws
       $ ln -s /{{folder/installed}}/aws-cli/aws_completer /{{usr/local/bin}}/aws_completer
       ```

        If you don't have write permissions to the folder, you may need to use `sudo` in your command. The following example uses `sudo` with the default location for a symlink in `/usr/local/bin/`:

       ```
       $ sudo ln -s /{{folder/installed}}/aws-cli/aws /{{usr/local/bin}}/aws
       $ sudo ln -s /{{folder/installed}}/aws-cli/aws_completer /{{usr/local/bin}}/aws_completer
       ```
**Note**  
You can view debug logs for the installation by pressing **Cmd\+L** anywhere in the installer. This opens a log pane that enables you to filter and save the log. The log file is also automatically saved to `/var/log/install.log`.

1. To verify that the shell can find and run the `aws` command in your `$PATH`, use the following commands. 

   ```
   $ which aws
   /usr/local/bin/aws 
   $ aws --version
   aws-cli/2.27.41 Python/3.11.6 Darwin/23.3.0
   ```

   If the `aws` command cannot be found, you might need to restart your terminal or follow the troubleshooting in [Troubleshooting errors for the AWS CLI](cli-chap-troubleshooting.md).

------
#### [ Command line installer - All users ]

If you have `sudo` permissions, you can install the AWS CLI for all users on the computer. We provide the steps in one easy to copy and paste group. See the descriptions of each line in the following steps. 

```
$ curl "https://awscli.amazonaws.com/AWSCLIV2.pkg" -o "AWSCLIV2.pkg"
$ sudo installer -pkg AWSCLIV2.pkg -target /
```

**Guided installation instructions**

1. Download the file using the `curl` command. The `-o` option specifies the file name that the downloaded package is written to. In this example, the file is written to `AWSCLIV2.pkg` in the current folder.

   ```
   $ curl "https://awscli.amazonaws.com/AWSCLIV2.pkg" -o "AWSCLIV2.pkg"
   ```

1. Run the standard macOS `installer` program, specifying the downloaded `.pkg` file as the source. Use the `-pkg` parameter to specify the name of the package to install, and the `-target /` parameter for which drive to install the package to. The files are installed to `/usr/local/aws-cli`, and a symlink is automatically created in `/usr/local/bin`. You must include `sudo` on the command to grant write permissions to those folders. 

   ```
   $ sudo installer -pkg ./AWSCLIV2.pkg -target /
   ```

   After installation is complete, debug logs are written to `/var/log/install.log`.

1. To verify that the shell can find and run the `aws` command in your `$PATH`, use the following commands. 

   ```
   $ which aws
   /usr/local/bin/aws 
   $ aws --version
   aws-cli/2.27.41 Python/3.11.6 Darwin/23.3.0
   ```

   If the `aws` command cannot be found, you might need to restart your terminal or follow the troubleshooting in [Troubleshooting errors for the AWS CLI](cli-chap-troubleshooting.md).

------
#### [ Command line - Current user ]

1. To specify which folder the AWS CLI is installed to, you must create an XML file with any file name. This file is an XML-formatted file that looks like the following example. Leave all values as shown, except you must replace the path {{/Users/myusername}} in line 9 with the path to the folder you want the AWS CLI installed to. *The folder must already exist, or the command fails.* The following XML example, named {{choices.xml}}, specifies the installer to install the AWS CLI in the folder `/Users/myusername`, where it creates a folder named `aws-cli`.

   ```
   <?xml version="1.0" encoding="UTF-8"?>
   <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
   <plist version="1.0">
     <array>
       <dict>
         <key>choiceAttribute</key>
         <string>customLocation</string>
         <key>attributeSetting</key>
         <string>{{/Users/myusername}}</string>
         <key>choiceIdentifier</key>
         <string>default</string>
       </dict>
     </array>
   </plist>
   ```

1. Download the `pkg` installer using the `curl` command. The `-o` option specifies the file name that the downloaded package is written to. In this example, the file is written to `AWSCLIV2.pkg` in the current folder.

   ```
   $ curl "https://awscli.amazonaws.com/AWSCLIV2.pkg" -o "AWSCLIV2.pkg"
   ```

1. Run the standard macOS `installer` program with the following options:
   + Specify the name of the package to install by using the `-pkg` parameter.
   + Specify installing to a *current user only* by setting the `-target` parameter to `CurrentUserHomeDirectory`.
   + Specify the path (relative to the current folder) and name of the XML file that you created in the `-applyChoiceChangesXML` parameter.

   The following example installs the AWS CLI in the folder `/Users/myusername/aws-cli`.

   ```
   $ installer -pkg AWSCLIV2.pkg \
               -target CurrentUserHomeDirectory \
               -applyChoiceChangesXML {{choices.xml}}
   ```

1. Because standard user permissions typically don't allow writing to folders in your `$PATH`, the installer in this mode doesn't try to add the symlinks to the `aws` and `aws_completer` programs. For the AWS CLI to run correctly, you must manually create the symlinks after the installer finishes. If your `$PATH` includes a folder you can write to and you specify the folder as the target's path, you can run the following command without `sudo`. If you don't have a writable folder in your `$PATH`, you must use `sudo` for permissions to write to the specified target folder. The default location for a symlink is `/usr/local/bin/`. Replace `folder/installed` with the path to your AWS CLI installation.

   ```
   $ sudo ln -s /{{folder/installed}}/aws-cli/aws /{{usr/local/bin}}/aws
   $ sudo ln -s /{{folder/installed}}/aws-cli/aws_completer /{{usr/local/bin}}/aws_completer
   ```

   After installation is complete, debug logs are written to `/var/log/install.log`.

1. To verify that the shell can find and run the `aws` command in your `$PATH`, use the following commands. 

   ```
   $ which aws
   /usr/local/bin/aws 
   $ aws --version
   aws-cli/2.27.41 Python/3.11.6 Darwin/23.3.0
   ```

   If the `aws` command cannot be found, you might need to restart your terminal or follow the troubleshooting in [Troubleshooting errors for the AWS CLI](cli-chap-troubleshooting.md).

------

### Windows
<a name="install-windows"></a>

#### Install requirements
<a name="install-windows-prereqs"></a>
+ We support the AWS CLI on Microsoft-supported versions of 64-bit Windows.
+ Admin rights to install software (if installing for all users).

#### Install the AWS CLI
<a name="install-windows-instructions"></a>

You can install the AWS CLI on Windows for all users (requires admin rights) or for only the current user (no admin rights required). The install script is the recommended method. We update the AWS CLI regularly; to see the latest version, see the [AWS CLI version 2 Changelog](https://raw.githubusercontent.com/aws/aws-cli/v2/CHANGELOG.rst) on *GitHub*. To update an existing installation, see [AWS CLI update instructions](#cli-update).

------
#### [ Install script (recommended) ]

The install script downloads, verifies, and installs the AWS CLI for Windows in one step. It installs for the current user (`%LOCALAPPDATA%\Programs\Amazon\AWSCLIV2`) by default.

**To install** the AWS CLI, run the following command from a PowerShell prompt.

```
C:\> irm https://awscli.amazonaws.com/v2/install.ps1 | iex
```

To install for all users instead (installs under `Program Files`), which requires an elevated (Administrator) PowerShell prompt, download the script to a file first, then run it with the `-System` argument.

```
C:\> irm 'https://awscli.amazonaws.com/v2/install.ps1' -OutFile install.ps1; try { .\install.ps1 -System } finally { Remove-Item install.ps1 }
```

For the full list of options, run the script with `-Help` using the same pattern.

```
C:\> irm 'https://awscli.amazonaws.com/v2/install.ps1' -OutFile install.ps1; try { .\install.ps1 -Help } finally { Remove-Item install.ps1 }
```

To update the AWS CLI, see [AWS CLI update instructions](#cli-update).

------
#### [ MSI installer - All users ]

Use the `AWSCLIV2.msi` installer to install the AWS CLI for all users on the machine. This installation type requires admin rights.

1. Download and run the AWS CLI MSI installer for Windows (64-bit):

   [https://awscli.amazonaws.com/AWSCLIV2.msi](https://awscli.amazonaws.com/AWSCLIV2.msi) 

   Alternatively, you can run the `msiexec` command to run the MSI installer.

   ```
   C:\> msiexec.exe /i https://awscli.amazonaws.com/AWSCLIV2.msi
   ```

   For various parameters that can be used with `msiexec`, see [msiexec](https://docs.microsoft.com/en-us/windows-server/administration/windows-commands/msiexec) on the *Microsoft Docs* website. For example, you can use the `/qn` flag for a silent installation.

   ```
   C:\> msiexec.exe /i https://awscli.amazonaws.com/AWSCLIV2.msi {{/qn}}
   ```

1. To confirm the installation, open the **Start** menu, search for `cmd` to open a command prompt window, and at the command prompt use the `aws --version` command. 

   ```
   C:\> aws --version
   aws-cli/2.27.41 Python/3.11.6 Windows/10 exe/AMD64 prompt/off
   ```

   If Windows is unable to find the program, you might need to close and reopen the command prompt window to refresh the path, or follow the troubleshooting in [Troubleshooting errors for the AWS CLI](cli-chap-troubleshooting.md).

------
#### [ MSI installer - Current user ]

Use the `AWSCLIV2-User.msi` installer to install the AWS CLI for only the current user. This installation type does not require admin rights. The install commands and AWS CLI functionality are the same as the all-users installation.

1. Download and run the AWS CLI MSI installer for Windows (64-bit):

   [https://awscli.amazonaws.com/AWSCLIV2-User.msi](https://awscli.amazonaws.com/AWSCLIV2-User.msi) 

   Alternatively, you can run the `msiexec` command to run the MSI installer.

   ```
   C:\> msiexec.exe /i https://awscli.amazonaws.com/AWSCLIV2-User.msi
   ```

   For various parameters that can be used with `msiexec`, see [msiexec](https://docs.microsoft.com/en-us/windows-server/administration/windows-commands/msiexec) on the *Microsoft Docs* website. For example, you can use the `/qn` flag for a silent installation.

   ```
   C:\> msiexec.exe /i https://awscli.amazonaws.com/AWSCLIV2-User.msi {{/qn}}
   ```

1. To confirm the installation, open the **Start** menu, search for `cmd` to open a command prompt window, and at the command prompt use the `aws --version` command. 

   ```
   C:\> aws --version
   aws-cli/2.27.41 Python/3.11.6 Windows/10 exe/AMD64 prompt/off
   ```

   If Windows is unable to find the program, you might need to close and reopen the command prompt window to refresh the path, or follow the troubleshooting in [Troubleshooting errors for the AWS CLI](cli-chap-troubleshooting.md).

------

## AWS CLI update instructions
<a name="cli-update"></a>

If you installed the AWS CLI using the install script, an official installer (the Linux command line installer, the macOS PKG installer, or the Windows MSI installer), you can update to the latest version by running the following command.

```
$ aws update
```

If you installed the AWS CLI for all users, run `aws update` from an elevated shell (`sudo` on Linux and macOS, or an Administrator PowerShell prompt on Windows). `aws update` preserves whether your existing installation is for all users or the current user.

## Troubleshooting AWS CLI install and uninstall errors
<a name="install-tshoot"></a>

If you come across issues after installing or uninstalling the AWS CLI, see [Troubleshooting errors for the AWS CLI](cli-chap-troubleshooting.md) for troubleshooting steps. For the most relevant troubleshooting steps, see [Command not found errors](cli-chap-troubleshooting.md#tshoot-install-not-found), [The "`aws --version`" command returns a different version than you installed](cli-chap-troubleshooting.md#tshoot-install-wrong-version), and [The "`aws --version`" command returns a version after uninstalling the AWS CLI](cli-chap-troubleshooting.md#tshoot-uninstall-1).

## Next steps
<a name="install-next-steps"></a>

After you successfully install the AWS CLI, you can safely delete your downloaded installer files. After completing the steps in [Prerequisites to use the AWS CLI version 2](getting-started-prereqs.md) and installing the AWS CLI, you should perform a [Setting up the AWS CLI](getting-started-quickstart.md).
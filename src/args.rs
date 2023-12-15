use clap:: {
    Args,
    Parser,
    Subcommand
};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct IsoMdlArgs {
    /// subcommand is either "issue" or "verify"
    #[clap(subcommand)]
    pub subcommand: IsoMdlCommand,
}

#[derive(Debug, Subcommand)]
pub enum IsoMdlCommand {
    /// Issue an mdoc based mDL based on a provided input json file
    Issue(IssueCommand),
    /// Verify an mdoc based mDL based on a provided input mdoc file
    Verify(VerifyCommand),
    /// Run issue on the supplied input json file and then verify the result
    IssueVerify(IssueVerifyCommand)
}

#[derive(Debug, Args)]
pub struct IssueCommand {
    /// input is a json file that supplies the claims values to be in the issued mdoc-based mDL
    pub input_filename: String,
    /// optional output file is issued mdoc , if not specified, then standard out is used
    pub output_filename: Option<String>
}

#[derive(Debug, Args)]
pub struct VerifyCommand {
    /// input is a mdoc-based mDL file to be verified
    pub input_filename: String,
    /// optional output file is parsed claims from verified mdoc, if not specified, then standard out is used
    pub output_filename: Option<String>
}

#[derive(Debug, Args)]
pub struct IssueVerifyCommand {
    /// input is a json file that supplies the claims values to be in the issued mdoc-based mDL
    pub input_filename: String,
    /// optional output file is parsed claims from verified mdoc, if not specified, then standard out is used
    pub output_filename: Option<String>
}


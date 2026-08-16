use radiacode_nuclides::NuclideId;

pub enum SpectrumPlotAction {
    OpenCatalogue(NuclideId),
    OpenCatalogueChain(NuclideId),
}

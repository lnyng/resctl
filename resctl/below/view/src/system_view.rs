// Copyright (c) Facebook, Inc. and its affiliates.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use cursive::view::{Identifiable, View};
use cursive::views::{LinearLayout, TextView};
use cursive::Cursive;

use crate::ViewState;

mod render_impl {
    use std::collections::BTreeMap;

    use cursive::utils::markup::StyledString;
    use once_cell::sync::Lazy;

    use crate::render::ViewItem;

    use base_render::render_config as rc;
    use model::{Queriable, SingleDiskModel, SingleNetModel, SystemModel};

    /// Renders corresponding Fields From SystemModel.
    type SystemViewItem = ViewItem<model::SystemModelFieldId>;

    static SYS_CPU_ITEMS: Lazy<Vec<SystemViewItem>> = Lazy::new(|| {
        use model::SingleCpuModelFieldId::{SystemPct, UsagePct, UserPct};
        use model::SystemModelFieldId::Cpu;
        vec![
            ViewItem::from_default(Cpu(UsagePct)),
            ViewItem::from_default(Cpu(UserPct)),
            ViewItem::from_default(Cpu(SystemPct)),
        ]
    });

    static SYS_MEM_ITEMS: Lazy<Vec<SystemViewItem>> = Lazy::new(|| {
        use model::MemoryModelFieldId::{Anon, File, Free, Total};
        use model::SystemModelFieldId::Mem;
        vec![
            ViewItem::from_default(Mem(Total)),
            ViewItem::from_default(Mem(Free)),
            ViewItem::from_default(Mem(Anon)),
            ViewItem::from_default(Mem(File)),
        ]
    });

    static SYS_VM_ITEMS: Lazy<Vec<SystemViewItem>> = Lazy::new(|| {
        use model::SystemModelFieldId::Vm;
        use model::VmModelFieldId::{PgpginPerSec, PgpgoutPerSec, PswpinPerSec, PswpoutPerSec};
        vec![
            ViewItem::from_default(Vm(PgpginPerSec)),
            ViewItem::from_default(Vm(PgpgoutPerSec)),
            ViewItem::from_default(Vm(PswpinPerSec)),
            ViewItem::from_default(Vm(PswpoutPerSec)),
        ]
    });

    const ROW_NAME_WIDTH: usize = 15;
    const ROW_FIELD_NAME_WIDTH: usize = 9;
    const ROW_FIELD_WIDTH: usize = 17;

    pub fn render_row<T: Queriable>(
        name: &'static str,
        model: &T,
        items: impl Iterator<Item = ViewItem<T::FieldId>>,
    ) -> StyledString {
        let mut row = StyledString::new();
        row.append(base_render::get_fixed_width(name, ROW_NAME_WIDTH));
        for item in items {
            let title = item.config.render_config.get_title();
            row.append(base_render::get_fixed_width(title, ROW_FIELD_NAME_WIDTH));
            row.append(item.update(rc!(width(ROW_FIELD_WIDTH))).render(model));
        }
        row
    }

    pub fn render_models_row<'a, T: 'a + Queriable>(
        name: &'static str,
        models: impl Iterator<Item = (&'a String, &'a T)>,
        item: ViewItem<T::FieldId>,
    ) -> StyledString {
        let item = item.update(rc!(width(ROW_FIELD_WIDTH)));
        let mut row = StyledString::new();
        row.append(base_render::get_fixed_width(name, ROW_NAME_WIDTH));
        for (name, model) in models {
            row.append(base_render::get_fixed_width(name, ROW_FIELD_NAME_WIDTH));
            row.append(item.render(model));
        }
        row
    }

    pub fn render_cpu_row(model: &SystemModel) -> StyledString {
        render_row("CPU", model, SYS_CPU_ITEMS.iter().cloned())
    }

    pub fn render_mem_row(model: &SystemModel) -> StyledString {
        render_row("Mem", model, SYS_MEM_ITEMS.iter().cloned())
    }

    pub fn render_vm_row(model: &SystemModel) -> StyledString {
        render_row("VM", model, SYS_VM_ITEMS.iter().cloned())
    }

    pub fn render_io_row(disks: &BTreeMap<String, SingleDiskModel>) -> StyledString {
        use model::SingleDiskModelFieldId::DiskTotalBytesPerSec;
        render_models_row(
            "I/O",
            disks.iter().filter(|(_, sdm)| sdm.minor == Some(0)),
            ViewItem::from_default(DiskTotalBytesPerSec),
        )
    }

    pub fn render_iface_row(ifaces: &BTreeMap<String, SingleNetModel>) -> StyledString {
        use model::SingleNetModelFieldId::ThroughputPerSec;
        render_models_row(
            "Iface",
            ifaces.iter(),
            ViewItem::from_default(ThroughputPerSec),
        )
    }
}

fn fill_content(c: &mut Cursive, v: &mut LinearLayout) {
    let view_state = &c
        .user_data::<ViewState>()
        .expect("No data stored in Cursive object!");

    let system_model = view_state.system.borrow();
    let network_model = view_state.network.borrow();
    let cpu_row = render_impl::render_cpu_row(&system_model);
    let mem_row = render_impl::render_mem_row(&system_model);
    let vm_row = render_impl::render_vm_row(&system_model);
    let io_row = render_impl::render_io_row(&system_model.disks);
    let iface_row = render_impl::render_iface_row(&network_model.interfaces);

    let mut view = LinearLayout::vertical();
    view.add_child(TextView::new(cpu_row));
    view.add_child(TextView::new(mem_row));
    view.add_child(TextView::new(vm_row));
    view.add_child(TextView::new(io_row));
    view.add_child(TextView::new(iface_row));

    *v = view;
}

pub fn refresh(c: &mut Cursive) {
    let mut v = c
        .find_name::<LinearLayout>("system_view")
        .expect("No system_view view found!");

    fill_content(c, &mut v);
}

pub fn new(c: &mut Cursive) -> impl View {
    let mut view = LinearLayout::vertical();
    fill_content(c, &mut view);
    view.with_name("system_view")
}

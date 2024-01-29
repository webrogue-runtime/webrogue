package com.webrogue;

import android.content.Intent;
import android.support.v7.widget.RecyclerView;
import android.view.LayoutInflater;
import android.view.MenuItem;
import android.view.View;
import android.view.ViewGroup;
import android.widget.PopupMenu;
import android.widget.Switch;
import android.widget.TextView;
import android.widget.Toast;

/**
 * Provide views to RecyclerView with data from mDataSet.
 */
public class ModListAdapter extends RecyclerView.Adapter<ModListAdapter.ViewHolder> {
    private static final String TAG = "CustomAdapter";

    private WebrogueLauncherWorker worker;

    // BEGIN_INCLUDE(recyclerViewSampleViewHolder)
    /**
     * Provide a reference to the type of views that you are using (custom ViewHolder)
     */
    public static class ViewHolder extends RecyclerView.ViewHolder {
        private final TextView nameTextView;
        private final TextView sizeTextView;
        private final Switch activeSwitch;
        private final View view;

        public ViewHolder(View v) {
            super(v);
            // Define click listener for the ViewHolder's View.
            view = v;
            nameTextView = (TextView) v.findViewById(R.id.mod_name_text_view);
            sizeTextView = (TextView) v.findViewById(R.id.mod_size_text_view);
            activeSwitch = (Switch) v.findViewById(R.id.active_switch);
        }

        public void configure(WebrogueLauncherWorker.Mod mod, WebrogueLauncherWorker worker, int position, ModListAdapter adapter) {
            nameTextView.setText(mod.name);
            sizeTextView.setText(toNumInUnits(mod.file.length()));
            activeSwitch.setChecked(mod.isActive);
            activeSwitch.setOnCheckedChangeListener((compoundButton, b) -> { worker.setModActive(mod, b); });
            view.setOnClickListener(new View.OnClickListener() {
                @Override
                public void onClick(View v) {
                    PopupMenu popup = new PopupMenu(v.getContext(), v);
                    //Inflating the Popup using xml file
                    popup.getMenuInflater().inflate(R.menu.mod_item_row, popup.getMenu());

                    //registering popup with OnMenuItemClickListener
                    popup.setOnMenuItemClickListener(new PopupMenu.OnMenuItemClickListener() {
                        public boolean onMenuItemClick(MenuItem item) {
                            String id = v.getContext().getResources().getResourceName(item.getItemId()).split("\\/")[1];
                            switch (id) {
                                case "delete":
                                    worker.delete(mod);
                                    worker.refresh();
                                    adapter.notifyItemRemoved(position);
                                    break;
                                default:
                            }
                            return true;
                        }
                    });

                    popup.show();//showing popup menu
                }
            });
        }

        public static String toNumInUnits(long bytes) {
            int u = 0;
            double value = bytes;
            for (; value > 1024; value /= 1024) {
                u++;
            }

            return String.format("%.1f %cB", value, " kMGTPE".charAt(u));
        }
    }
    // END_INCLUDE(recyclerViewSampleViewHolder)

    public ModListAdapter(WebrogueLauncherWorker worker) {
        this.worker = worker;
    }

    // BEGIN_INCLUDE(recyclerViewOnCreateViewHolder)
    // Create new views (invoked by the layout manager)
    @Override
    public ViewHolder onCreateViewHolder(ViewGroup viewGroup, int viewType) {
        // Create a new view.
        View v = LayoutInflater.from(viewGroup.getContext())
                .inflate(R.layout.mod_row_item, viewGroup, false);

        return new ViewHolder(v);
    }
    // END_INCLUDE(recyclerViewOnCreateViewHolder)

    // BEGIN_INCLUDE(recyclerViewOnBindViewHolder)
    // Replace the contents of a view (invoked by the layout manager)
    @Override
    public void onBindViewHolder(ViewHolder viewHolder, final int position) {
        //Log.d(TAG, "Element " + position + " set.");

        // Get element from your dataset at this position and replace the contents of the view
        // with that element
        viewHolder.configure(worker.getMod(position), worker, position, this);
    }
    // END_INCLUDE(recyclerViewOnBindViewHolder)

    // Return the size of your dataset (invoked by the layout manager)
    @Override
    public int getItemCount() {
        return worker.getModCount();
    }
}
